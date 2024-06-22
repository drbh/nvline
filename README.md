# nvline

![Crates.io Version](https://img.shields.io/crates/v/nvline)
![Crates.io Downloads](https://img.shields.io/crates/d/nvline)

your nvidia gpu usage timeline.

```bash
cargo install nvline
```

### Features

- record `nvidia-smi` output to a file
- analyze the timeline [https://huggingface.co/spaces/drbh/nvline](https://huggingface.co/spaces/drbh/nvline)

the following command will record `100` lines of `nvidia-smi` output every `1000` milliseconds to `log1.jsonl`.

```bash
nvline --max-lines 100 --interval 1000 --file-path log1.jsonl
# 2024-06-21T04:20:58.576466Z  INFO nvline: name=NVIDIA A10G device=0 used=4 percent=4/23028 (0.02%)
# 2024-06-21T04:20:59.635555Z  INFO nvline: name=NVIDIA A10G device=0 used=4 percent=4/23028 (0.02%)
# 2024-06-21T04:21:00.691742Z  INFO nvline: name=NVIDIA A10G device=0 used=4 percent=4/23028 (0.02%)
# 2024-06-21T04:21:01.751656Z  INFO nvline: name=NVIDIA A10G device=0 used=4 percent=4/23028 (0.02%)
```

logfile looks like

```json
{"driver_version":"545.23.08","index":0,"memory_free":22508,"memory_total":23028,"memory_used":4,"name":"NVIDIA A10G","temperature_gpu":30,"timestamp":1718943658}
{"driver_version":"545.23.08","index":0,"memory_free":22508,"memory_total":23028,"memory_used":4,"name":"NVIDIA A10G","temperature_gpu":30,"timestamp":1718943659}
{"driver_version":"545.23.08","index":0,"memory_free":22508,"memory_total":23028,"memory_used":4,"name":"NVIDIA A10G","temperature_gpu":30,"timestamp":1718943660}
{"driver_version":"545.23.08","index":0,"memory_free":22508,"memory_total":23028,"memory_used":4,"name":"NVIDIA A10G","temperature_gpu":30,"timestamp":1718943661}
```

### cli options

```bash
nvline --help`
Usage: nvline [OPTIONS]

Options:
  -m, --max-lines <MAX_LINES>  Maximum number of lines to keep in the log file [default: 100]
  -i, --interval <INTERVAL>    Interval between log entries in milliseconds [default: 1000]
  -f, --file-path <FILE_PATH>  Path to the log file [default: gpu_log.jsonl]
  -h, --help                   Print help
  -V, --version                Print version

```
