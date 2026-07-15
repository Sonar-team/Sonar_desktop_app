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

## Environment variables reference

All timing instrumentation is compiled in only under the `capture_timing`
Cargo feature (`src-tauri/Cargo.toml`) — it never runs, and these variables
have no effect, on a normal build.

| Variable | Applies to | Default | Effect |
| --- | --- | --- | --- |
| `SONAR_CAPTURE_TIMING_LOG` | live capture + PCAP import | OS-specific app log dir, `capture-timing-<pid>.jsonl` | Path of the JSONL file both instrumentations append to. |
| `SONAR_CAPTURE_TIMING_RUN_ID` | live capture | `capture-<pid>-<unix_ns>` | Run id prefix; SONAR appends `-run01`, `-run02`... per capture started in the same process. |
| `SONAR_CAPTURE_TIMING_SAMPLE_RATE` | live capture | `100` (1 packet in 100) | Sampling rate for `capture_pipeline_timing`; every packet still counts toward `capture_run_summary`/`capture_packet_batch_ipc_timing`, only the per-packet pipeline breakdown is sampled. |
| `SONAR_IMPORT_TIMING_SAMPLE_RATE` | PCAP import | falls back to `SONAR_CAPTURE_TIMING_SAMPLE_RATE`, else `1` (every packet) | Sampling rate for `import_packet_timing`. |

Default log path if `SONAR_CAPTURE_TIMING_LOG` is unset:
- Linux: `$XDG_DATA_HOME/fr.sonar.ssf/logs/capture-timing-<pid>.jsonl` (or
  `~/.local/share/...`)
- Windows: `%LOCALAPPDATA%\fr.sonar.ssf\logs\capture-timing-<pid>.jsonl`
- macOS: `~/Library/Logs/fr.sonar.ssf/capture-timing-<pid>.jsonl`

## Event reference

Written by the live capture pipeline
(`src-tauri/src/state/capture/capture_handle/threads/capture_timing.rs`):

| Event | When | Key fields |
| --- | --- | --- |
| `capture_pipeline_timing` | per sampled packet | `parse_l2_ns`/`parse_l3_ns`/`parse_l4_ns`/`parse_l7_ns`/`parse_total_ns`, `packet_owned_ns`, `label_lookup_ns`, `matrix_update_ns`, `graph_update_ns`, `graph_ipc_ns`, `graph_updates`, `graph_ipc_failures`, `pipeline_total_ns` |
| `capture_packet_batch_ipc_timing` | per `PacketBatch` sent to the frontend | `batch_len`, `batch_max`, `batch_interval_ms`, `batch_full`, `ipc_ns`, `ok` |
| `capture_run_summary` | once, when the capture stops | `packet_total`, `avg_packets_per_second`, `batch_count`, `full_batch_count`, `packet_batch_ipc_avg_ns`/`p95_ns`/`p99_ns`, `active_duration_ns`, `pool_small_allocated`, `pool_large_allocated`, `pool_allocated_bytes`, `pool_exhausted` |

Written by PCAP import (`src-tauri/src/commandes/import/timing.rs` +
`pcap.rs`):

| Event | When | Key fields |
| --- | --- | --- |
| `import_packet_timing` | per sampled packet | same pipeline breakdown as `capture_pipeline_timing` |
| `import_parse_error_timing` | per unparseable packet | error context |
| `import_file_timing` | once per imported file | `file_path`, packet/error counts, total duration |
| `import_snapshot_timing` | once, graph snapshot sent at the end of import | duration of building/sending the final `GraphSnapshot` |

All durations are nanoseconds (`_ns` suffix); all timestamps
(`ts_unix_ns`) are Unix epoch nanoseconds.

## Quick CLI reading without Grafana

```bash
# Full run summary
grep '"event":"capture_run_summary"' observability/data/sonar-logs/sonar-timing.jsonl | jq .

# Average per pipeline stage across the whole run
jq -s '[.[] | select(.event=="capture_pipeline_timing")]
  | {parse: (map(.parse_total_ns)|add/length),
     matrix: (map(.matrix_update_ns)|add/length),
     graph: (map(.graph_update_ns)|add/length),
     graph_ipc: (map(.graph_ipc_ns)|add/length)}' \
  observability/data/sonar-logs/sonar-timing.jsonl
```

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
