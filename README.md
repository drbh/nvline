# nvline

your nvidia gpu usage timeline

```bash
make run
# 2024-06-21T04:20:58.576466Z  INFO nvline: name=NVIDIA A10G device=0 used=4 percent=4/23028 (0.02%)
# 2024-06-21T04:20:59.635555Z  INFO nvline: name=NVIDIA A10G device=0 used=4 percent=4/23028 (0.02%)
# 2024-06-21T04:21:00.691742Z  INFO nvline: name=NVIDIA A10G device=0 used=4 percent=4/23028 (0.02%)
# 2024-06-21T04:21:01.751656Z  INFO nvline: name=NVIDIA A10G device=0 used=4 percent=4/23028 (0.02%)
```

data is dumped from `nvidia-smi` every second to `gpu_log.jsonl` in a circular buffer fashion, never exceeding `1MB`.

The logfile looks like

```json
{"driver_version":"545.23.08","index":0,"memory_free":22508,"memory_total":23028,"memory_used":4,"name":"NVIDIA A10G","temperature_gpu":30,"timestamp":1718943658}
{"driver_version":"545.23.08","index":0,"memory_free":22508,"memory_total":23028,"memory_used":4,"name":"NVIDIA A10G","temperature_gpu":30,"timestamp":1718943659}
{"driver_version":"545.23.08","index":0,"memory_free":22508,"memory_total":23028,"memory_used":4,"name":"NVIDIA A10G","temperature_gpu":30,"timestamp":1718943660}
{"driver_version":"545.23.08","index":0,"memory_free":22508,"memory_total":23028,"memory_used":4,"name":"NVIDIA A10G","temperature_gpu":30,"timestamp":1718943661}
```
