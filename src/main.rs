use clap::Parser;
use serde::Serialize;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::process::Command;
use std::str;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Maximum number of lines to keep in the log file
    #[arg(short, long, default_value_t = 100)]
    max_lines: usize,

    /// Interval between log entries in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    interval: u64,

    /// Path to the log file
    #[arg(short, long, default_value = "gpu_log.jsonl")]
    file_path: String,
}

#[derive(Debug, Serialize, Clone)]
struct GpuInfo {
    index: usize,
    name: String,
    driver_version: String,
    memory_total: u64,
    memory_used: u64,
    memory_free: u64,
    temperature_gpu: u64,
}

fn parse_gpu_info(output: &str) -> Option<Vec<GpuInfo>> {
    let lines: Vec<&str> = output.trim().split('\n').collect();
    let mut all_gpu_info: Vec<GpuInfo> = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let data: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if data.len() < 6 {
            continue;
        }
        all_gpu_info.push(GpuInfo {
            index,
            name: data[0].to_string(),
            driver_version: data[1].to_string(),
            memory_total: data[2].replace(" MiB", "").parse().ok()?,
            memory_used: data[3].replace(" MiB", "").parse().ok()?,
            memory_free: data[4].replace(" MiB", "").parse().ok()?,
            temperature_gpu: data[5].parse().ok()?,
        });
    }

    Some(all_gpu_info)
}

fn parse_sysctl_vm_info(output: &str) -> Option<Vec<GpuInfo>> {
    let lines: Vec<&str> = output.trim().split('\n').collect();
    let mut mem_info: Vec<GpuInfo> = Vec::new();

    for line in lines {
        let data: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
        if data.len() < 2 {
            continue;
        }
        let key = data[0];
        let value = data[1];
        if key == "Total Memory" {
            mem_info.push(GpuInfo {
                index: 0,
                name: "Memory".to_string(),
                driver_version: "0".to_string(),
                memory_total: value.parse().ok()?,
                memory_used: 0,
                memory_free: 0,
                temperature_gpu: 0,
            });
        } else if key == "Used Memory" {
            mem_info[0].memory_used = value.parse().ok()?;
        } else if key == "Free Memory" {
            mem_info[0].memory_free = value.parse().ok()?;
        }
    }

    Some(mem_info)
}

#[derive(Debug, PartialEq, Eq)]
enum GpuInfoKind {
    Nvidia,
    VmStat,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let kind = if cfg!(target_os = "macos") {
        GpuInfoKind::VmStat
    } else {
        GpuInfoKind::Nvidia
    };

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(true)
        .open(&args.file_path)
        .expect("Failed to open log file");

    tracing_subscriber::fmt::init();

    let mut curr = 0;

    loop {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let output = if kind == GpuInfoKind::Nvidia {
            Command::new("nvidia-smi")
            .args([
                "--query-gpu=name,driver_version,memory.total,memory.used,memory.free,temperature.gpu",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to execute nvidia-smi command. Please ensure that nvidia-smi is installed and accessible in your PATH.",
                )
            })?
        } else {
            Command::new("sh")
            .arg("-c")
            .arg(r#"
                # Get memory info
                mem_info=$(sysctl -n hw.memsize vm.page_free_count vm.page_speculative_count)
                
                # Extract values
                total_mem=$(echo $mem_info | awk '{print $1}')
                free_pages=$(echo $mem_info | awk '{print $2}')
                speculative_pages=$(echo $mem_info | awk '{print $3}')
                
                # Calculate memory in bytes
                page_size=$(getconf PAGE_SIZE)
                free_mem=$((($free_pages + $speculative_pages) * $page_size))
                used_mem=$(($total_mem - $free_mem))
                
                # Print results
                echo "Total Memory: $total_mem"
                echo "Free Memory:  $free_mem"
                echo "Used Memory:  $used_mem"
            "#)
            .output()
            .map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to execute sysctl command. Please ensure that sysctl is installed and accessible in your PATH.",
                )
            })?
        };

        if output.status.success() {
            let output_str = str::from_utf8(&output.stdout).expect("Failed to parse output");

            let gpu_result = if kind == GpuInfoKind::Nvidia {
                parse_gpu_info(output_str)
            } else {
                parse_sysctl_vm_info(output_str)
            };

            if let Some(gpu_infos) = gpu_result {
                for gpu_info in gpu_infos {
                    let log_entry = json!({
                        "timestamp": timestamp,
                        "index": gpu_info.index,
                        "name": gpu_info.name,
                        "driver_version": gpu_info.driver_version,
                        "memory_total": gpu_info.memory_total,
                        "memory_used": gpu_info.memory_used,
                        "memory_free": gpu_info.memory_free,
                        "temperature_gpu": gpu_info.temperature_gpu,
                    });

                    let log_entry_str = format!("{}\n", log_entry);
                    let log_entry_bytes = log_entry_str.as_bytes();

                    let file_length = file.metadata()?.len();
                    let entry_length = log_entry_bytes.len() as u64;

                    let seek_pos = if file_length >= args.max_lines as u64 * entry_length {
                        curr as u64 * entry_length
                    } else {
                        file_length
                    };
                    file.seek(SeekFrom::Start(seek_pos))?;
                    if file_length >= args.max_lines as u64 * entry_length {
                        curr = (curr + 1) % args.max_lines;
                    }

                    file.write_all(log_entry_bytes)?;

                    tracing::info!(
                        "name={device} device={index} used={memory_used} percent={memory_used}/{memory_total} ({percent:.2}%)",
                        device = gpu_info.name,
                        index = gpu_info.index,
                        memory_used = gpu_info.memory_used,
                        memory_total = gpu_info.memory_total,
                        percent = gpu_info.memory_used as f32 / gpu_info.memory_total as f32 * 100.0
                    );
                }
            } else {
                eprintln!("Failed to parse GPU info");
            }
        } else {
            eprintln!("nvidia-smi command failed");
        }

        thread::sleep(Duration::from_millis(args.interval));
    }
}
