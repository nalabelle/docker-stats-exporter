use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize, Debug, Clone)]
pub struct DockerContainerStats {
    //"BlockIO":"1.1GB / 473MB"
    #[serde(rename = "BlockIO")]
    pub block_io: String,
    // "CPUPerc":"0.41%",
    #[serde(rename = "CPUPerc")]
    pub cpu_perc: String,
    // "Container":"9db408e1b7b7",
    #[serde(rename = "Container")]
    pub container: String,
    // "ID":"9db408e1b7b7",
    #[serde(rename = "ID")]
    pub id: String,
    // "MemPerc":"69.07%",
    #[serde(rename = "MemPerc")]
    pub mem_perc: String,
    //"MemUsage":"707.3MiB / 1GiB",
    #[serde(rename = "MemUsage")]
    pub mem_usage: String,
    //"Name":"paperless",
    #[serde(rename = "Name")]
    pub name: String,
    //"NetIO":"13.4MB / 2.32MB",
    #[serde(rename = "NetIO")]
    pub net_io: String,
    //"PIDs":"79"
    #[serde(rename = "PIDs")]
    pub pids: String,
}

const DOCKER_FORMAT: &str = r#"{{json .}}"#;

pub fn stats() -> Result<Vec<DockerContainerStats>> {
    let output = Command::new("docker")
        .args(&["stats", "--format", DOCKER_FORMAT, "--no-stream"])
        .output()?;

    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;

    if !output.status.success() {
        eprintln!(
            "`docker stats` returned non-zero exit code with output: \n{}\n{}",
            stdout, stderr
        );
        return Err(anyhow!("Docker stats command did bad :("));
    }

    let json_list_content = stdout.lines().collect::<Vec<&str>>().join(",");
    let json_string = format!("[{}]", json_list_content);

    let result = serde_json::from_str::<Vec<DockerContainerStats>>(json_string.as_str())?;
    Ok(result)
}
