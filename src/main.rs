use serde::Serialize;
use serde_json::json;
use std::collections::VecDeque;
use std::fs::OpenOptions;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::process::Command;
use std::str;
use std::thread;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

const MAX_LINES: usize = 10;
const INTERVAL: u64 = 1_000;
const FILE_PATH: &str = "gpu_log.jsonl";

#[derive(Debug, Serialize, Clone)]
struct GpuInfo {
    index: usize,
    name: String,
    driver_version: String,
    memory_total: u32,
    memory_used: u32,
    memory_free: u32,
    temperature_gpu: u32,
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

fn main() {
    let mut log: VecDeque<GpuInfo> = VecDeque::new();
    let file_path = FILE_PATH;

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .truncate(true)
        .open(file_path)
        .expect("Failed to open log file");

    tracing_subscriber::fmt::init();

    let mut curr = 0;

    loop {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();

        let output = Command::new("nvidia-smi")
            .arg("--query-gpu=name,driver_version,memory.total,memory.used,memory.free,temperature.gpu")
            .arg("--format=csv,noheader,nounits")
            .output()
            .expect("Failed to execute nvidia-smi");

        if output.status.success() {
            let output_str = str::from_utf8(&output.stdout).expect("Failed to parse output");
            if let Some(gpu_infos) = parse_gpu_info(output_str) {
                for gpu_info in &gpu_infos {
                    log.push_back(gpu_info.clone());

                    if log.len() > MAX_LINES {
                        log.pop_front();
                    }

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

                    let file_length = file.metadata().expect("Failed to get file metadata").len();
                    let entry_length = log_entry_bytes.len() as u64;

                    let seek_pos = if file_length >= MAX_LINES as u64 * entry_length {
                        curr as u64 * entry_length
                    } else {
                        file_length
                    };
                    file.seek(SeekFrom::Start(seek_pos))
                        .expect("Failed to seek");
                    if file_length >= MAX_LINES as u64 * entry_length {
                        curr = (curr + 1) % MAX_LINES;
                    }

                    file.write_all(log_entry_bytes)
                        .expect("Failed to write to file");

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

        thread::sleep(Duration::from_millis(INTERVAL));
    }
}
