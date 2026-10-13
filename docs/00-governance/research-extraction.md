# Research extraction matrix

!!! info "Phase 0 assessment · 2026-10-13"
    Extracted from CHRONOS-P0-001 v1.0. Assessment complete; engineering decisions pending review. Source intent, reviewer findings and approvals remain distinct.

The matrix consolidates repeated significant claims while retaining page locations. Document mappings are completed in Output 4. `R` means source requirement; `P` means proposal; `T` means unverified target/claim; `D` means clear research direction; `H` means historical/contextual material. These are authority labels, not validation results.

| ID | Kind / authority | Extracted item, paraphrased | BASE-001 pages |
| --- | --- | --- | --- |
| RES-001 | Context / H | FDAP lowers database development barriers; rapid production-readiness claim | 1–2 |
| RES-002 | Architecture / D | Flight, DataFusion, Arrow and Parquet as reusable foundation | 2, 11–13 |
| RES-003 | Architecture / H | Decoupled cloud compute/storage; S3/GCS and Iceberg examples | 2 |
| RES-004 | Capability / T | Broad open-format interoperability without translation costs | 2, 7 |
| RES-005 | Context / H | IoT/finance/observability demand and crowded TSDB market | 2–3 |
| RES-006 | Algorithm / H | Edge compaction using Gorilla/Chimp/Sprintz; negligible CPU and 10–20× claim | 3–4 |
| RES-007 | Capability / H | Edge raw/rollup retention and bandwidth-aware tiering | 4 |
| RES-008 | Commercial / H | Industrial/automotive/aerospace; open-core fleet control plane | 4–6 |
| RES-009 | Objective / D | Financial DX; mid-market funds, risk analysts, quants and prop desks | 6–8, 14, 18 |
| RES-010 | Capability / P | ASOF match between asynchronous trade and quote streams | 6, 16, 28 |
| RES-011 | Capability / P | TIME_BUCKET, OHLCV and tick-count windows | 6, 17, 28 |
| RES-012 | Requirement / R | Bitemporal state-of-knowledge access (source FR-3) | 6–7, 17, 28 |
| RES-013 | Capability / P | Python/Polars/Pandas/Jupyter through Flight | 6–7, 17, 27–28 |
| RES-014 | Target / T | Absolute zero-copy/no network parsing or serialization overhead | 3, 6, 10, 14, 20, 23 |
| RES-015 | Requirement / R | Self-hosted/private VPC/co-location and no internet dependency (NFR-3) | 7, 10, 17 |
| RES-016 | Architecture / P | Two-tier hot memory and streaming cold path | 8–9, 15–16 |
| RES-017 | Algorithm / P | Preallocated lock-free/Disruptor Arrow ring; concurrent queries | 8–9, 11–13, 16, 28–32 |
| RES-018 | Architecture / P | Micro-Parquet sequential background persistence | 9, 16, 27 |
| RES-019 | Architecture / P | Unified hot/cold catalog and query routing | 9, 12–13, 16, 27 |
| RES-020 | Requirement / R | No GC pauses and fully deterministic memory (NFR-2) | 10, 17, 35 |
| RES-021 | Capability / P | Reconstruct L1/L2/L3 order books at a requested timestamp | 10 |
| RES-022 | Commercial / T | Save engineering time; throughput/memory-based value pricing | 7–8, 10, 18 |
| RES-023 | Objective / D | Time-to-alpha, accessible trade data infrastructure and no data lock-in | 14–15 |
| RES-024 | Architecture / D | Single-node Rust daemon with embedded DataFusion | 13, 16, 26 |
| RES-025 | Capability / P | Binance/Coinbase WebSockets and ITCH structures; FIX mentioned architecturally | 8, 13, 16, 26 |
| RES-026 | Algorithm / P | Five-second vs ten-second/500k tick flush triggers | 9, 16, 27 |
| RES-027 | Requirement / R | Up to 1M ticks/s on eight cores (FR-1) | 17 |
| RES-028 | Requirement / R | ANSI SQL plus financial extensions (FR-2) | 17 |
| RES-029 | Requirement / R | P99 live-window queries below 1 ms (NFR-1) | 17 |
| RES-030 | Requirement / R | Late data/corrections without entire-history Parquet rewrites | 17 |
| RES-031 | Commercial / P | Open core; paid fleet, active-passive HA and SLAs | 17–18 |
| RES-032 | Target / T | Notebook ingestion 20× PostgreSQL/CSV | 17 |
| RES-033 | Target / T | ClickHouse merge-jitter, DuckDB contention and serialized-client comparisons | 18–20 |
| RES-034 | Architecture / P | EPYC/NVMe/ConnectX reference hardware | 21 |
| RES-035 | Algorithm / P | Core isolation/pinning and DPDK ingestion | 21–22 |
| RES-036 | Target / T | 2.5M sustained ticks/s at average 48-byte payload | 22 |
| RES-037 | Target / T | 8M ticks/s microburst for 500 ms | 23 |
| RES-038 | Target / T | P95 below 180 µs and P99 below 850 µs | 23 |
| RES-039 | Target / T | Flight egress approximately 4.2 GB/s | 23 |
| RES-040 | Architecture / D | Nanosecond timestamp representation throughout FDAP | 23–26 |
| RES-041 | Target / T | Nanoseconds necessary/sufficient for event sequence reconstruction | 23–25 |
| RES-042 | Target / T | Eight-byte ns values, signed epoch range and vectorized operations | 24–25 |
| RES-043 | Roadmap / P | Nine-month hot/hybrid/DX staged plan | 26–28 |
| RES-044 | Algorithm / P | Allocation audit after warmup using tools such as jemalloc | 26 |
| RES-045 | Target / T | Stage 1 1M ticks/s with concurrent query every 100 ms | 26–27 |
| RES-046 | Algorithm / P | Bounded ingestion micro-sort for out-of-order ticks | 27 |
| RES-047 | Target / T | Correct cross-tier aggregate completes below 50 ms | 27 |
| RES-048 | Target / T | 50M joined rows to remote Polars below 1.5 s | 27–28 |
| RES-049 | Algorithm / P | System-knowledge metadata for corrections and historical views | 28 |
| RES-050 | Algorithm / P | Cache padding and optional raw memory copying | 28–29, 32 |
| RES-051 | Prototype / P | SPSC builder rotator with sequence counters and unsafe Sync | 29–32 |
| RES-052 | Target / T | Builder finish preserves capacity and returns buffers automatically | 28, 31–32, 35, 38–39 |
| RES-053 | Algorithm / P | Full ring: spin/drop/panic alternatives | 31 |
| RES-054 | Prototype / P | TableProvider polls/drains batches during scan; projected MemoryExec | 33–34 |
| RES-055 | Prototype / P | 1,024 slots × 4,096 rows; simulated writer; 10 ms query loop | 34–35 |
| RES-056 | Target / T | Immutable batch integration guarantees safety and zero-allocation turnover | 35 |
| RES-057 | Algorithm / P | Optional timer publication under low-volume traffic | 32, 36, 39, 42 |
| RES-058 | Prototype / P | Double-buffer slot with atomic active index and unsafe Sync | 36–39 |
| RES-059 | Algorithm / P | Example flip every 500 µs or 1,000 rows; rolling hot window question | 36, 39 |
| RES-060 | Prototype / P | Disruptor batch-ready events and busy-spin consumer | 39–41 |
| RES-061 | Target / T | Sequencer protects buffers until DataFusion query completion | 41–42 |
| RES-062 | Target / T | Sub-100 ns signals and universal 64-byte alignment protection | 42 |
| RES-063 | Context / H | Numbered references lack a resolvable bibliography in the PDF | 1–13, 18–20, 39–42 |

## Page coverage ledger

| Physical pages | Material reviewed | Extraction coverage |
| --- | --- | --- |
| 1–2 | Contents, landscape and reusable stack | RES-001–005, 063 |
| 3–5 | Edge pivot, algorithms, verticals and fleet model | RES-006–008, 014 |
| 6–7 | Financial DX and primitives | RES-009–015, 022 |
| 8–10 | Hot/cold architecture and HFT positioning | RES-014–022, 025 |
| 11–13 | Build/borrow boundaries and MVP strategy | RES-002, 017, 019, 024–025, 063 |
| 14–15 | Draft PRD, objectives and diagram | RES-009, 014, 016, 023 |
| 16–17 | MVP, FR/NFRs, risks and adoption | RES-010–020, 024–032 |
| 18–20 | Enterprise tier and competitive matrix | RES-009, 022, 031, 033, 063 |
| 21–22 | Hardware/topology and sustained target | RES-034–036 |
| 23–25 | Burst/query/egress targets and time representation | RES-037–042 |
| 26–28 | Roadmap, validation goals and low-level strategies | RES-043–050, 052 |
| 29–32 | SPSC rotator prototype | RES-050–053, 057 |
| 33–35 | TableProvider and example integration | RES-052, 054–056 |
| 36–39 | Double buffering and timer alternatives | RES-057–060 |
| 40–42 | Disruptor blueprint and guarantees | RES-060–063 |

Coverage means reviewed and classified; it does not mean validated. Conversational prompts and repetitions are consolidated rather than treated as additional requirements.

## Source and related records

- [Unified Phase 0 assessment](../assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md) — preserved supplied document.
- [Research baseline](research-baseline.md) · [Assessment](research-assessment.md) · [Extraction matrix](research-extraction.md)
- [Technical validation](technical-validation.md) · [Assumptions & gaps](assumptions-and-gaps.md) · [Decision register](decision-register.md)
- [Performance profiles](performance-profiles.md) · [Traceability](traceability.md) · [Readiness & handoff](readiness-and-handoff.md)
