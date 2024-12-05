use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use prometheus::{
    core::{AtomicF64, AtomicU64, GenericGaugeVec},
    Opts,
};
use serde::Serialize;
use std::{collections::HashMap, num::ParseFloatError};

use crate::docker::DockerContainerStats;

// Percents have decimals and prometheus wants these as 64s instead of smaller.
type Percent = f64;
type Count = u64;
type Bytes = u64;

#[derive(Serialize, Debug, Clone)]
pub struct ContainerMetrics {
    pub name: String,
    pub id: String,
    pub container: String,
    pub pids: Count,
    pub cpu_usage_percent: Percent,
    pub mem_usage_percent: Percent,
    pub block_input_bytes: Bytes,
    pub block_output_bytes: Bytes,
    pub mem_usage_bytes: Bytes,
    pub mem_limit_bytes: Bytes,
    pub net_input_bytes: Bytes,
    pub net_output_bytes: Bytes,
}

impl ContainerMetrics {
    pub fn new(stat: &DockerContainerStats) -> Self {
        let (block_input_bytes, block_output_bytes) =
            parse_slashy_values(stat.block_io.as_str()).unwrap();
        let (mem_usage_bytes, mem_limit_bytes) =
            parse_slashy_values(stat.mem_usage.as_str()).unwrap();
        let (net_input_bytes, net_output_bytes) =
            parse_slashy_values(stat.net_io.as_str()).unwrap();

        let metrics = ContainerMetrics {
            name: stat.name.clone(),
            id: stat.id.clone(),
            container: stat.container.clone(),
            pids: stat.pids.parse::<Count>().unwrap(),
            cpu_usage_percent: parse_percent(stat.cpu_perc.clone()).unwrap(),
            mem_usage_percent: parse_percent(stat.mem_perc.clone()).unwrap(),
            block_input_bytes,
            block_output_bytes,
            mem_usage_bytes,
            mem_limit_bytes,
            net_input_bytes,
            net_output_bytes,
        };

        metrics
    }

    pub fn set_gauges(&self, gauges: &Measurements) {
        let labels = &self.labels();
        gauges
            .cpu_usage_percent
            .with_label_values(labels)
            .set(self.cpu_usage_percent);

        gauges
            .mem_usage_percent
            .with_label_values(labels)
            .set(self.mem_usage_percent);

        gauges
            .mem_usage_bytes
            .with_label_values(labels)
            .set(self.mem_usage_bytes);

        gauges
            .mem_limit_bytes
            .with_label_values(labels)
            .set(self.mem_limit_bytes);

        gauges
            .block_input_bytes
            .with_label_values(labels)
            .set(self.block_input_bytes);

        gauges
            .block_output_bytes
            .with_label_values(labels)
            .set(self.block_output_bytes);

        gauges
            .net_input_bytes
            .with_label_values(labels)
            .set(self.net_input_bytes);

        gauges
            .net_output_bytes
            .with_label_values(labels)
            .set(self.net_output_bytes);
    }

    pub fn labels(&self) -> [&str; 3] {
        [
            self.name.as_str(),
            self.id.as_str(),
            self.container.as_str(),
        ]
    }
}

lazy_static! {
    static ref UNIT_MAP: HashMap<&'static str, f64> = {
        let mut map = HashMap::new();
        map.insert("B", 1f64);
        map.insert("kB", 1000f64);
        map.insert("MB", 1000f64 * 1000f64);
        map.insert("GB", 1000f64 * 1000f64 * 1000f64);
        map.insert("TB", 1000f64 * 1000f64 * 1000f64 * 1000f64);
        map
    };
}

pub fn convert_to_bytes(value: f64, unit: String) -> Result<Bytes> {
    let Some(conversion_rate) = UNIT_MAP.get(unit.as_str()) else {
        return Err(anyhow!(
            "Couldn't convert unit '{}' to bytes, that was weird..",
            unit
        ));
    };

    // Avoid fractional bytes
    let result = (conversion_rate * value).round() as u64;
    Ok(result)
}

fn parse_percent(mut percent_string: String) -> Result<Percent, ParseFloatError> {
    percent_string.pop();
    percent_string.parse::<Percent>()
}

fn parse_bytes(str: String) -> Result<Bytes> {
    let backwards_unit = str
        .chars()
        .rev()
        .take_while(|c| c.is_alphabetic())
        .collect::<String>();
    let unit = backwards_unit.chars().rev().collect::<String>();
    let index = str.len() - unit.len();
    let value = &str[0..index];
    let float_value = value.parse::<f64>()?;
    let result = convert_to_bytes(float_value, unit)?;
    Ok(result)
}

fn parse_slashy_values(str: &str) -> Result<(Bytes, Bytes)> {
    let values: Vec<Bytes> = str
        .split(" / ")
        .map(|value| parse_bytes(value.to_string()))
        .collect::<Result<Vec<Bytes>, _>>()?;
    if values.len() != 2 {
        return Err(anyhow!("Bad string: {}", str));
    }
    Ok((values[0], values[1]))
}

// fn percent_gauge(opts: Opts, value: Percent) -> Result<GenericGauge<AtomicF64>> {
//     let gauge = Gauge::new(name.replace("-", "_"), help)?;
//     gauge.set(value as f64);
//     Ok(gauge)
// }
//
// fn get_gauge(name: String, value: , help: String, value: f63) -> Result<GenericGauge<AtomicF64>> {
//     let gauge = Gauge::new(name.replace("-", "_"), help)?;
//     gauge.set(value);
//     Ok(gauge)
// }

const LABELS: [&str; 3] = ["name", "id", "container"];

pub struct Measurements {
    pub pid_total: GenericGaugeVec<AtomicU64>,
    pub cpu_usage_percent: GenericGaugeVec<AtomicF64>,
    pub mem_usage_percent: GenericGaugeVec<AtomicF64>,
    pub mem_usage_bytes: GenericGaugeVec<AtomicU64>,
    pub mem_limit_bytes: GenericGaugeVec<AtomicU64>,
    pub block_input_bytes: GenericGaugeVec<AtomicU64>,
    pub block_output_bytes: GenericGaugeVec<AtomicU64>,
    pub net_input_bytes: GenericGaugeVec<AtomicU64>,
    pub net_output_bytes: GenericGaugeVec<AtomicU64>,
}

impl Measurements {
    pub fn new() -> Self {
        Self {
            pid_total: GenericGaugeVec::new(
                Opts::new("pid_count", "PIDs running in the container"),
                &LABELS,
            )
            .unwrap(),
            cpu_usage_percent: GenericGaugeVec::new(
                Opts::new("cpu_usage_percent", "CPU usage for the container"),
                &LABELS,
            )
            .unwrap(),
            mem_usage_percent: GenericGaugeVec::new(
                Opts::new("mem_usage_percent", "MEM usage for the container"),
                &LABELS,
            )
            .unwrap(),
            mem_usage_bytes: GenericGaugeVec::new(
                Opts::new("mem_usage_bytes", "MEM usage for the container"),
                &LABELS,
            )
            .unwrap(),
            mem_limit_bytes: GenericGaugeVec::new(
                Opts::new("mem_limit_bytes", "MEM limit for the container"),
                &LABELS,
            )
            .unwrap(),
            block_input_bytes: GenericGaugeVec::new(
                Opts::new("block_input_bytes", "Block input for the container"),
                &LABELS,
            )
            .unwrap(),
            block_output_bytes: GenericGaugeVec::new(
                Opts::new("block_output_bytes", "Block output for the container"),
                &LABELS,
            )
            .unwrap(),
            net_input_bytes: GenericGaugeVec::new(
                Opts::new("net_input_bytes", "Network input for the container"),
                &LABELS,
            )
            .unwrap(),
            net_output_bytes: GenericGaugeVec::new(
                Opts::new("net_output_bytes", "Network output for the container"),
                &LABELS,
            )
            .unwrap(),
        }
    }

    pub fn register(&self, registry: &prometheus::Registry) -> Result<()> {
        registry.register(Box::new(self.pid_total.clone()))?;
        registry.register(Box::new(self.cpu_usage_percent.clone()))?;
        registry.register(Box::new(self.mem_usage_percent.clone()))?;
        registry.register(Box::new(self.mem_usage_bytes.clone()))?;
        registry.register(Box::new(self.mem_limit_bytes.clone()))?;
        registry.register(Box::new(self.block_input_bytes.clone()))?;
        registry.register(Box::new(self.block_output_bytes.clone()))?;
        registry.register(Box::new(self.net_input_bytes.clone()))?;
        registry.register(Box::new(self.net_output_bytes.clone()))?;
        Ok(())
    }
}
