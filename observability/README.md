# SONAR observability stack

This stack runs Grafana, Loki and Promtail for SONAR timing JSONL files.

## Start
```bash
docker compose -f docker-compose.observability.yml up -d
```

Grafana is available at <http://localhost:3000>.

Default credentials:
- user: `admin`
- password: `admin`

## Feed SONAR timing logs
Point SONAR timing output to the directory read by Promtail:

```bash
SONAR_CAPTURE_TIMING_LOG="$PWD/observability/data/sonar-logs/sonar-timing.jsonl" \
SONAR_CAPTURE_TIMING_RUN_ID="capture-$(date +%Y%m%d-%H%M%S)" \
SONAR_IMPORT_TIMING_SAMPLE_RATE=1 \
deno task tauri dev --features capture_timing
```

Promtail also reads `/tmp/sonar-*.jsonl` through a read-only bind mount, which
is useful for quick local profiling runs such as `/tmp/sonar-import-dhcp.jsonl`.

For live capture profiling, use `SONAR_CAPTURE_TIMING_SAMPLE_RATE` instead of
or in addition to `SONAR_IMPORT_TIMING_SAMPLE_RATE`.

`SONAR_CAPTURE_TIMING_RUN_ID` is optional. SONAR treats it as a run prefix and
adds a per-capture suffix such as `-run01`, `-run02`, so several captures in the
same app process stay separate in Grafana. If it is omitted, SONAR generates a
prefix from the process id and current timestamp.

The `SONAR Capture Run Summary` dashboard reads `capture_run_summary` events
written at the end of each capture run. It shows packets, average packets/s, IPC
batch count, PacketBatch IPC avg/p95/p99, full batch count and batch interval by
`run_id`.

## Queries
Promtail adds these labels:
- `job="sonar-timing"`
- `app="sonar"`
- `event="<json event>"`
- `run_id="<capture run id>"` when present
- `file_path="<pcap path>"` when present

Useful LogQL examples:

```logql
{job="sonar-timing"} | json
```

```logql
{job="sonar-timing", event="import_file_timing"} | json
```

```logql
avg_over_time({job="sonar-timing", event="import_packet_timing"} | json | unwrap pipeline_total_ns [5m]) / 1000000
```
