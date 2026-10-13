# Performance profiles & feasibility

!!! info "Phase 0 assessment · 2026-10-13"
    Extracted from CHRONOS-P0-001 v1.0. Assessment complete; engineering decisions pending review. Source intent, reviewer findings and approvals remain distinct.

## Performance and scheduling ledger

Every entry below is a source target or illustrative setting, not a measured result. Profiles must not be combined into one acceptance promise.

| Profile | Source-stated value | Conditions in source | Missing acceptance details | Source |
| --- | --- | --- | --- | --- |
| PERF-MVP-ING | Up to 1,000,000 ticks/s continuously | Standard 8-core bare metal | Tick schema; duration; losses; adapter; query load | p. 17 |
| PERF-MVP-Q | P99 < 1 ms | Live in-memory window aggregates | Window/rows; concurrency; measurement endpoints; freshness | p. 17 |
| PERF-REF-ING | 2,500,000 ticks/s sustained | EPYC profile; average 48-byte L2 tick | Wire vs resident bytes; duration; durability; workload | pp. 21–22 |
| PERF-REF-BURST | Up to 8,000,000 ticks/s for up to 500 ms | Same reference; preallocated ring | Baseline load; drain rate; queue occupancy; loss policy | p. 23 |
| PERF-REF-Q | P95 < 180 µs; P99 < 850 µs | Time-bucket/VWAP on hot data under simultaneous load | Query definition; sample count; timer; tail methodology | p. 23 |
| PERF-REF-EGRESS | Approximately 4.2 GB/s | Loopback/100GbE wording | Separate local/remote tests; schema; compression; client endpoint | p. 23 |
| PERF-STAGE-1 | 1M ticks/s and query every 100 ms | Stage 1 exit gate | Duration, dataset and safety verification | pp. 26–27 |
| PERF-STAGE-2 | Hybrid aggregate < 50 ms | Cross disk/memory boundary | Rows; cold bytes; cache state; concurrency | p. 27 |
| PERF-STAGE-3 | 50M rows into remote Polars < 1.5 s | Four-line SQL containing ASOF JOIN | Row width; result size; join workload; network and timing | pp. 27–28 |
| PERF-DX | Python loading 20× faster | PostgreSQL/CSV comparison | Comparable baseline, versions, workloads | p. 17 |
| PERF-SIGNAL | Often < 100 ns signaling | Busy-spin library claim | Platform, contention, sample method | p. 42 |
| FLUSH-A | Every 5 s or 500,000 ticks | Illustrative cold-path explanation | Trigger precedence and durability | p. 9 |
| FLUSH-B | Every 10 s or 500,000 ticks | MVP and Stage 2 | Visibility separate from persistence | pp. 16, 27 |
| PUBLISH-A | 500 µs or 1,000 rows | Double-buffer example | Timer owner and missed interval behavior | p. 36 |
| PUBLISH-B | Optional 500 µs timer; 4,096-row batches | Query integration discussion | Not an implemented timer | pp. 34–36 |
| QUERY-EXAMPLE | Query loop every 10 ms | Simulated integration code | Distinct from Stage 1 gate cadence | p. 35 |
| ROADMAP | Months 1–3 hot; 4–6 hybrid; 7–9 DX | Proposed three-stage roadmap | Staffing, dependencies and estimates | pp. 26–28 |

## Reference hardware, retained without endorsement

BASE-001 p. 21 specifies EPYC 9354 (32 cores/64 threads, 3.25 GHz base/3.8 GHz boost, 256 MB L3); 384 GB DDR5-4800 as 6×64 GB; 960 GB OS/log NVMe; 2×3.84 TB striped cold-path NVMe; ConnectX-6 Dx dual-port 25/100GbE with DPDK. Pages 21–22 allocate cores 0–1 to OS/logging/flushing, 2–5 to ingestion and 6–31 to DataFusion/Flight.

The six-module configuration is not full population of the processor's twelve memory channels [EXT-03]. Single socket does not by itself establish uniform locality. Striping is not a recovery contract. Sharing two cores between OS/logging/flushing may create contention. These are qualification findings, not replacement hardware requirements.

## Quantitative feasibility checks (reviewer-derived)

These calculations are preliminary checks, not new acceptance criteria. Decimal units are used and metadata/wire/allocator costs are excluded unless noted.

| Check | Derivation | Implication |
| --- | --- | --- |
| Sustained payload | 2.5M × 48 bytes = 120 MB/s = 10.368 TB/day | Cold disk/retention/compression sizing needs explicit assumptions |
| Burst payload | 8M × 0.5 s × 48 bytes = 192 MB | Incoming burst bytes, not necessarily backlog |
| Excess backlog | If service remains 2.5M/s, (8M−2.5M) × 0.5 × 48 = 132 MB | Illustrative minimum additional buffer; resident bytes and headroom still missing |
| Example ring | 1,024 × 4,096 = 4,194,304 rows | At 1M/s roughly 4.19 s capacity without drain; not five minutes of retention |
| Example logical values | 4,194,304 × (8+8+4) = 83,886,080 bytes ≈ 80 MiB | Three-column sample differs from 48-byte target; double buffers and outstanding batches increase memory |
| Five-minute reference window | 2.5M × 300 × 48 = 36 GB | Hot retention requires a separate budget from the ingestion queue |
| Threshold-driven flush | 500k / 1M = 0.5 s; 500k / 2.5M = 0.2 s | Row threshold dominates the nominal 10-second timer at these rates |
| Remote result rate | 50M × 48 / 1.5 = 1.6 GB/s | Conditional result width; exceeds nominal 10GbE payload ceiling but may fit 25/100GbE before overhead |

The two 3.84 TB drives provide 7.68 TB nominal raw capacity when striped. At 120 MB/s raw payload, that is about 17.8 hours before compression, filesystem overhead, reserves and redundancy. Neither compression ratio nor sustainable end-to-end write throughput is proven. The 4.2 GB/s egress target and 1.5-second result target describe different tests and are not directly interchangeable.

## Related evidence

Hardware qualifications reference `EXT-03` in the [evidence register](research-baseline.md#evidence-register-and-maintenance). Benchmark closure is tracked by GAP-013 and DEC-005; hardware decisions by DEC-011.

## Source and related records

- [Unified Phase 0 assessment](../assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md) — preserved supplied document.
- [Research baseline](research-baseline.md) · [Assessment](research-assessment.md) · [Extraction matrix](research-extraction.md)
- [Technical validation](technical-validation.md) · [Assumptions & gaps](assumptions-and-gaps.md) · [Decision register](decision-register.md)
- [Performance profiles](performance-profiles.md) · [Traceability](traceability.md) · [Readiness & handoff](readiness-and-handoff.md)
