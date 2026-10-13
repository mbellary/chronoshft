# ChronosHFT — Phase 0 Research Baseline Assessment

| Control field | Value |
| --- | --- |
| Document ID | CHRONOS-P0-001 |
| Version / date | 1.0 / 2026-10-13 |
| Status | Assessment complete; engineering decisions pending review |
| Source | HFT - Research Notes.pdf; 42 physical PDF pages; 582,331 bytes |
| Source title | ChronosHFT: High-Frequency Trading TSDB Research & PRD |
| Source status | Draft / Vision Realisation; October 2026; Founding Team (p. 14) |
| Source identifier | BASE-001 |
| Source SHA-256 | `18b5ed26b6f4dfd8a83c452abf7cb81946f338078e66c85c71f48343631f37a2` |
| Authority | User-designated research baseline; source proposals are not automatically approved designs |
| Intended format | Markdown suitable for the future GitHub MkDocs repository |
| Scope | All 42 source pages; five assessment outputs; rules, baseline profiles and readiness gates |

## 1. Assessment outcome

The dominant later proposal is a single-node, Rust financial time-series database for market-data capture and concurrent analytics, built around Flight, DataFusion, Arrow and Parquet (FDAP). Earlier edge/IoT commercialization material remains part of the source history. Neither that material nor occasional references to strategy/order routing establish an approved full trading-execution product boundary.

The source supports a coherent product direction but does not supply reproducible benchmark evidence, pinned dependency versions, executable complete examples, an approved ownership/reclamation protocol, or sufficient durability and recovery contracts. The original source remains intact. Corrections, gaps and recommended investigations in this assessment are separate reviewer findings.

Phase 0 extraction is complete. Phase 1 documentation scaffolding may proceed with these open items visible. Detailed implementation readiness has not been established. No performance claim is marked empirically validated and no ADR is approved by this document.

## 2. Rules and assessment method

1. **Preserve the baseline.** Retain BASE-001 unchanged, with its hash and page count. A revised source receives a new baseline ID and a documented delta.
2. **No silent requirements.** Extract source requirements as written in substance. Reviewer recommendations become gaps, validation actions or proposed decisions, never implicit product commitments.
3. **Separate existence from truth.** A source-stated requirement proves research intent, not feasibility or achievement. A code sample labelled production-grade is still unvalidated until its invariants and implementation are verified.
4. **Cite physical PDF pages.** `BASE-001 pp. X–Y` uses the PDF viewer page number, not repeated or inconsistent section numbering. Repeated claims share one extraction ID with all relevant pages.
5. **Paraphrase, preserve qualifiers.** Keep thresholds, units, hardware profiles, workloads and uncertainty. Do not convert “expected” metrics into measured results or “up to” into an unconditional sustained guarantee.
6. **Keep competing proposals.** Do not select the latest statement automatically. Distinct hardware, flush, query-cadence and roadmap profiles remain independently recorded until resolved.
7. **Use bounded validation.** Static inspection can identify missing protocols and direct code behavior. It cannot certify concurrency safety, runtime compatibility or latency. Record the evidence boundary explicitly.
8. **Separate approval from validation.** A technically supported fact is not a product approval; a product approval does not validate a benchmark.
9. **Keep bidirectional traceability.** Extraction → validation/gap/decision → target document → future requirement/design/test/evidence. Later artifacts must retain the originating extraction IDs.
10. **Version external evidence.** Record access date, upstream version/commit when known and scope. The source has no Cargo manifest or lockfile; live upstream documentation cannot establish compatibility of its samples.
11. **Do not import adjacent projects.** No architecture, contract scheme or runtime choice from other projects is adopted implicitly.
12. **Handle unknowns explicitly.** Use `not specified`, `unverified` or `decision pending`; do not fabricate owners, approvals, benchmark runs or dates.

### Classification vocabulary

| Dimension | Values and meaning |
| --- | --- |
| Extraction kind | Context; Objective; Requirement; Capability; Architecture; Algorithm; Prototype; Target; Commercial; Roadmap |
| Validation status | Validated: supported within stated evidence scope; Unverified: claim lacks evidence; Incorrect: stated claim contradicted by evidence or direct behavior; Incomplete: necessary contract/design is missing; Investigation: compatibility or feasibility needs dedicated work |
| Decision status | Source-established direction: clearly stated research intent; Proposed: technique/choice offered without approval; Unresolved: conflicting or unspecified choice |
| Disposition | Preserve; qualify; investigate; replace only through approved decision; defer scope |
| Priority | Critical: blocks safe/correct design; High: blocks measurable acceptance or production readiness; Medium: documentation, delivery or commercial uncertainty |

These identifiers are introduced for Phase 0, not claimed to exist in the PDF: `RES-NNN`, `VAL-NNN`, `GAP-NNN`, `CON-NNN`, `DEC-NNN`. IDs are stable and never reused. Register owners below are proposed roles, not assigned people. All open records have no committed due date.

### Work performed and evidence limits

The complete page-marked text was read, with local PDF extraction and visual inspection of the buffer-flip code on p. 38. The four prototype families were inspected statically. Selected primary-source checks were made for Arrow builder ownership, Flight transport and EPYC memory channels. No source code was compiled, no benchmark executed and no unsafe implementation certified. Broad competitive, market and protocol claims remain unverified.

## 3. Baseline inventory and distinct profiles

### 3.1 Source-established directions

| Area | Preserved research direction | Source | Qualification |
| --- | --- | --- | --- |
| Product | Developer-first real-time financial TSDB | pp. 6–17 | Full trading engine boundary unresolved |
| MVP deployment | Single-node Rust binary with embedded DataFusion | pp. 13, 16, 26 | HA and fleet management appear as paid/future features |
| Foundation | Arrow memory; DataFusion query; Parquet persistence; Flight clients | pp. 2, 11–16 | Version and extension boundaries unspecified |
| Data plane | Ingest → hot batches → background cold persistence; unified query | pp. 8–9, 15–16 | Visibility and handoff semantics unresolved |
| User experience | SQL; Python/Polars/Pandas/Jupyter; financial primitives | pp. 6–7, 16–17, 27–28 | Exact API/dialect and conversion behavior unresolved |
| Time | Nanosecond-resolution timestamps | pp. 23–26 | Resolution does not establish ordering or clock accuracy |
| Deployment environment | Private VPC/on-premises/co-location; offline operation | pp. 7, 10, 17, 21–22 | OS, security and operational contracts missing |
| Commercial direction | Open core; paid fleet/HA/support; throughput/resource bands | pp. 17–18 | License and commercial assumptions unapproved |

### 3.2 Performance and scheduling ledger

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

### 3.3 Reference hardware, retained without endorsement

BASE-001 p. 21 specifies EPYC 9354 (32 cores/64 threads, 3.25 GHz base/3.8 GHz boost, 256 MB L3); 384 GB DDR5-4800 as 6×64 GB; 960 GB OS/log NVMe; 2×3.84 TB striped cold-path NVMe; ConnectX-6 Dx dual-port 25/100GbE with DPDK. Pages 21–22 allocate cores 0–1 to OS/logging/flushing, 2–5 to ingestion and 6–31 to DataFusion/Flight.

The six-module configuration is not full population of the processor's twelve memory channels [EXT-03]. Single socket does not by itself establish uniform locality. Striping is not a recovery contract. Sharing two cores between OS/logging/flushing may create contention. These are qualification findings, not replacement hardware requirements.

## 4. Output 1 — Research Extraction Matrix

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

### Page coverage ledger

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

## 5. Output 2 — Technical Validation Register

| ID / related extraction | Status | Finding and evidence boundary | Required closure evidence |
| --- | --- | --- | --- |
| VAL-001 / RES-001 | Unverified | Fast assembly of primitives does not establish production readiness | Reliability/security/correctness evidence; qualify commercial wording |
| VAL-002 / RES-002 | Validated (role only) | FDAP roles are consistent with the source and Flight/Arrow primary documentation [EXT-01/02]; Chronos integration untested | Pin compatible versions and integration tests |
| VAL-003 / RES-006 | Unverified | Zero-CPU and 10–20× compression claims lack workloads; Arrow/Parquet encoding integration absent | Losslessness tests; compression/CPU benchmarks and format design |
| VAL-004 / RES-014 | Incorrect (absolute claim) | Flight uses gRPC and Arrow IPC [EXT-01]; no-serialization/no-network-parsing guarantee is too broad | Boundary-specific copy/serialization measurements and client matrix |
| VAL-005 / RES-020 | Incomplete | No tracing GC does not bound allocator, destruction, paging, scheduling or I/O latency | Define bounded path and allocation/deallocation/page-fault budgets |
| VAL-006 / RES-027,029,036–039,045,047–048 | Unverified | No measured runs, harness, hardware record or result artifacts | Reproducible workload-specific evidence |
| VAL-007 / RES-033 | Unverified | Categorical competitor shortcomings and example latency/loading comparisons lack reproducible evidence | Versioned feature audit and fair benchmark; no universal claims |
| VAL-008 / RES-034 | Incorrect (population claim) | Six DIMMs are described as fully populated; EPYC 9354 has twelve memory channels [EXT-03] | Corrected proposed BOM and motherboard population plan |
| VAL-009 / RES-034–035 | Incomplete | Single socket/pinning alone do not prove locality or jitter isolation; OS and flusher share cores | Record BIOS/NUMA/SMT/IRQ/affinity and measure interference |
| VAL-010 / RES-040,042 | Validated (bounded arithmetic) | Signed 64-bit ns is eight bytes and roughly ±292 years around epoch; unit precision is not clock accuracy | Boundary/overflow/timezone round-trip tests |
| VAL-011 / RES-041 | Incorrect (ordering inference) | Equal ns timestamps still possible; timestamps alone cannot guarantee feed event ordering | Sequence/partition/tie-break rules and replay tests |
| VAL-012 / RES-051,053 | Incomplete | Writer appends before full-ring check; failed rotate returns and later appends continue in the same slot; capacity bound unenforced | Full-queue policy before writes, overflow tests and protocol proof |
| VAL-013 / RES-051 | Incomplete (safety) | Public `&self` append/poll methods plus unsafe Sync do not enforce one producer/consumer; concurrent callers can alias mutable slots | Restricted handles and documented exclusion/publication invariants |
| VAL-014 / RES-052 | Incorrect for inspected upstream behavior | PrimitiveBuilder finish transfers its values using `mem::take`; capacity is not retained in that builder [EXT-02]. Batch drops do not invoke the source's reserve loop | Version-pinned allocation test and explicit pool/reclamation design |
| VAL-015 / RES-054 | Incorrect for persistent-table semantics | scan consumes the queue; later/concurrent scans may see different/disjoint batches. It is not a retained snapshot registry | Repeatable non-destructive scans and snapshot boundary tests |
| VAL-016 / RES-054 | Investigation | Projection is applied to each batch and passed again to MemoryExec with original schema; risks depend on API version | Nontrivial/reordered projections and schema consistency tests |
| VAL-017 / RES-055,060 | Incomplete | Example calls `schema()` and `row_count()` not defined in shown structs; no manifest/version pins | Compile complete examples against a pinned dependency set |
| VAL-018 / RES-040,051 | Incomplete | Code uses Int64 schema rather than Arrow Timestamp(ns); timezone/epoch semantics not encoded | Approved schema and Arrow/Parquet/Flight round-trip tests |
| VAL-019 / RES-058 | Incomplete (safety) | Atomic index flip does not enforce exclusive append/flush access or track safe reuse. No reader lease/retirement protocol is shown | Ownership state machine; concurrent-call tests; memory model review |
| VAL-020 / RES-058,056 | Incorrect (guarantee) | Moving builder storage into immutable arrays differs from querying shared mutable buffers. Either allocation or safe reclamation must be addressed; flip alone proves neither | Explicit transfer/reuse model with delayed readers |
| VAL-021 / RES-060–061 | Incorrect (unconditional guarantee) | Disruptor event completion is not necessarily DataFusion query completion if the callback hands off a batch | Query lifetime tied to buffer retirement; slow-reader tests |
| VAL-022 / RES-062 | Unverified | Busy-spin latency/alignment claims depend on crate version and platform | Verify source layout; measured signaling and CPU cost |
| VAL-023 / RES-019,030,046,049 | Incomplete | Hybrid visibility, late/corrected data and historical knowledge policies absent | Model and oracle tests covering handoff/corrections |
| VAL-024 / RES-018 | Incomplete | Parquet format/background writes alone do not establish crash-consistent durable acknowledgment | Failure model; commit/catalog protocol and restart tests |
| VAL-025 / RES-025,035 | Investigation | DPDK packet ingress and WebSocket application decoding are distinct integration problems | Adapter-specific transport, framing, TLS and allocation design |
| VAL-026 / RES-028 | Incomplete | “ANSI SQL” and financial functions lack a supported subset and precise semantics | Dialect/version and function conformance suite |
| VAL-027 / RES-021 | Incomplete | Order-book reconstruction lacks venue event/state rules and recovery policy | Venue-specific reference traces and deterministic oracle |
| VAL-028 / RES-063 | Incomplete | Numeric citations cannot be resolved to references from the PDF | Recover bibliography or mark each affected assertion unsupported |

### Critical prototype interpretation

The SPSC design must not be reported as intrinsically racy under every possible restricted use: its sequence pattern suggests an intended single producer/consumer discipline. The problem is that this discipline is not enforced by the exposed API and its capacity/full-ring behavior is incomplete. Similarly, `finish()` can detach array storage, so the double-buffer sample does not prove that a query reads builder memory being overwritten. The justified findings are missing caller exclusion, unsupported no-allocation/reuse claims, and absent lifetime-based reclamation. A stronger data-race conclusion requires a concrete supported concurrent-call schedule.

The TableProvider issue is direct: polling advances the shared read sequence. Query planning therefore consumes ingestion output rather than taking a stable view of a retained table. The Disruptor integration's callback only prints rows; it does not implement the claimed DataFusion registry or query-completion barrier.

## 6. Output 3 — Gap and Conflict Register

### 6.1 Gaps

All items are **Open**. Closure is evidence-based; proposed ownership does not grant approval authority.

| ID | Priority / proposed owner | Missing contract or evidence | Source link | Closure criterion |
| --- | --- | --- | --- | --- |
| GAP-001 | Critical / Product | TSDB versus complete trading engine boundary | RES-009,021,024,055 | Approved scope with exclusions and capability map |
| GAP-002 | Critical / Systems | Enforced producer/consumer handles and slot lifecycle | RES-017,051,058 | Design and safety invariants reviewed; adversarial tests |
| GAP-003 | Critical / Query | Stable snapshots, retention and slow-reader policy | RES-019,054,061 | Repeatable reads across publication/reclamation |
| GAP-004 | Critical / Ingestion | Full queue/backpressure/drop/disconnect policy | RES-037,053 | Explicit policy with loss accounting and bounded memory tests |
| GAP-005 | Critical / Storage | Durability point, acknowledgment, WAL choice and crash recovery | RES-018,030 | Approved failure model and injected-failure recovery evidence |
| GAP-006 | Critical / Domain | Sequence gaps, duplicates, ordering and deterministic replay | RES-021,025,041 | Venue/partition rules and replay oracle |
| GAP-007 | Critical / Storage+Query | Atomic hot/cold visibility, no gaps or double counting | RES-019,047 | Snapshot/catalog handoff and concurrency tests |
| GAP-008 | High / Domain | Fixed-point/decimal vs float prices; sizes, instruments and IDs | RES-025,051 | Financial schema/range/rounding contract |
| GAP-009 | High / Domain | ASOF ties/tolerance/direction; buckets, calendars, OHLCV null/empty rules | RES-010–011,028 | Query semantics and expected-output cases |
| GAP-010 | High / Storage | Corrections, lateness limits and bitemporal version semantics | RES-012,030,046,049 | Immutable correction/version rules; late-event tests |
| GAP-011 | High / Engineering | Dependency pins, APIs and runnable complete samples | RES-051,054,060 | Cargo lockfile; buildable examples; compatibility record |
| GAP-012 | High / Performance | Allocation boundaries, pool budgets and retained-reader memory | RES-017,020,044,052 | Measured allocations/deallocations and exhaustion policy |
| GAP-013 | High / Performance | Workload definitions and benchmark methodology | RES-027,029,036–039,045,047–048 | Versioned generators, scripts, results and acceptance contracts |
| GAP-014 | High / Operations | Clock domains, synchronization, timezone, overflow and provenance | RES-040–042 | Timestamp/sequence schema and clock-quality reporting |
| GAP-015 | High / Operations | AuthN/AuthZ, encryption, secrets and feed entitlements | RES-015,025 | Threat model and scoped controls; offline packaging |
| GAP-016 | High / Operations | Observability, start/stop/drain, disk-full and process failure | RES-018,031,035 | Runbooks and injected-failure tests |
| GAP-017 | High / Storage | Partition/pruning/catalog, compaction, retention and disk sizing | RES-018–019,030,034 | Storage lifecycle and capacity model |
| GAP-018 | High / Networking | Adapter protocols, TLS, reconnect, framing and feed recovery | RES-025,035 | Adapter-specific contracts and conformance traces |
| GAP-019 | Medium / Product | Validated personas, customer value and commercial assumptions | RES-009,022–023,031–033 | Evidence register; reviewed positioning and license scope |
| GAP-020 | Medium / Documentation | Bibliography, source versions and baseline change process | RES-063 | Recoverable references or explicit unsupported markers |
| GAP-021 | High / Reliability | HA replication consistency and failover boundary | RES-031 | Separate future-scope decision; if adopted, replication/recovery contract |
| GAP-022 | High / Engineering | Test tiers for unsafe/concurrent, financial and end-to-end behavior | RES-044–048,051,058 | Requirement-linked verification plan; tools chosen by applicability |

### 6.2 Conflicts and ambiguities

| ID | Priority | Preserved alternatives | Interpretation / resolution required |
| --- | --- | --- | --- |
| CON-001 | Critical | Edge/IoT pp. 3–5 versus financial TSDB pp. 6–17 | Earlier exploration is not silently promoted to MVP; DEC-001 |
| CON-002 | High | 8-core 1M pp. 17 vs 32-core 2.5M/8M pp. 21–23 | Distinct profiles, not necessarily contradiction; DEC-005 |
| CON-003 | High | 5 s flush p. 9 vs 10 s pp. 16/27; 500k ticks in both | Select configurable policy and distinguish persistence/publication; DEC-004 |
| CON-004 | Critical | Mutable Arrow views vs immutable finished arrays pp. 9/16/31/35/38 | Decide transfer/retain/recycle model; DEC-003 |
| CON-005 | Critical | Nonblocking writer guarantee vs finite ring and spin/drop/panic p. 31 | Cannot guarantee losslessness, bounded memory and unlimited producer progress simultaneously under stalled consumers; DEC-006 |
| CON-006 | High | Timestamp(ns) pp. 24–25 vs Int64 schema p. 29 | Semantic type mismatch; DEC-007 |
| CON-007 | High | 500 µs/1k rows p. 36 vs 4,096 rows p. 34 vs optional timer | Distinct examples, no canonical freshness contract; DEC-004 |
| CON-008 | High | 100 ms gate pp. 26–27 vs 10 ms query example p. 35 | Benchmark cadence versus sample behavior; DEC-005 |
| CON-009 | High | Near-term MVP financial features pp. 16–17 vs Stage 3 months 7–9 | Align MVP capability scope and roadmap; DEC-002 |
| CON-010 | High | Six DIMMs fully populated p. 21 vs twelve channels [EXT-03] | Correct proposed hardware population statement; DEC-011 |
| CON-011 | High | Arrow timezone None p. 24 vs UTC-adjusted Parquet p. 25 | Specify instant/local-time semantics and conversions; DEC-007 |
| CON-012 | Medium | Core open-source pp. 17–18 vs proprietary-layer language pp. 12–13 | Define license and component boundary; DEC-012 |

## 7. Output 4 — Documentation Mapping Matrix

The sixteen documents are target destinations, not documents created or approved by this assessment. DOC-01 is represented by this Phase 0 assessment; the immutable PDF remains BASE-001. Every RES ID is mapped at least once; duplication is intentional where concerns cross document boundaries.

| Document | Responsibility | Extraction IDs | Open-item inputs |
| --- | --- | --- | --- |
| DOC-01 Research Baseline & Assessment | Provenance, extraction and assessment | RES-001–063 | All registers |
| DOC-02 Product Requirements | Personas, scope, FR/NFRs and outcomes | RES-009–015,020–025,027–032,036–039,045,047–048 | GAP-001,013,019; CON-001/002/009 |
| DOC-03 Product Architecture | User capabilities and product boundaries | RES-002,009–025,031 | GAP-001,009,021 |
| DOC-04 Platform/System Architecture | Runtime topology and deployment | RES-002–003,015–019,024–025,034–035,060 | GAP-005,007,018; DEC-010/011 |
| DOC-05 Platform Design | Interfaces and lifecycle contracts | RES-016–019,025–026,030,046,049,051,053–061 | GAP-002–007,017 |
| DOC-06 Engineering Requirements | Measurable correctness/safety/performance | RES-012,014–015,020,027–030,036–042,044–048,052,056,061–062 | All Critical/High gaps |
| DOC-07 Engineering Design | Algorithms and component designs | RES-010–012,016–019,021,026,030,040,044,046,049–061 | VAL-012–027; GAP-002–012 |
| DOC-08 Rust Concepts & Patterns | Design-linked concepts and exercises | RES-017,020,024,044,050–062 | GAP-002,011–012,022 |
| DOC-09 Benchmark, Testing & Setup | Reproducible workloads and acceptance | RES-014,027,029,032–039,042,044–048,055–057,059,062 | GAP-013,022; DEC-005 |
| DOC-10 Market Data & Financial Semantics | Feeds, time, books and query semantics | RES-010–012,021,025,028,030,040–042,046,049 | GAP-006,008–010,014,018 |
| DOC-11 Data Model & Storage | Schemas, persistence and recovery | RES-002–004,007,016–019,026,030,034,040,046,049,051–052,058 | GAP-005,007–010,017 |
| DOC-12 Performance & Hardware | CPU, memory, network and profiling | RES-006,017,020,034–039,042,044,050,052,062 | VAL-008/009/022; GAP-012–013 |
| DOC-13 Reliability, Security & Operations | Failure behavior, controls and runbooks | RES-007–008,015,018,020–021,025,030–031,034–035,053 | GAP-004–006,014–018,021 |
| DOC-14 Architecture Decision Records | Approved choices and alternatives | RES-003,006–008,014,017,024–026,030–031,034–035,040,046,051–062 | DEC-001–014; all conflicts |
| DOC-15 Roadmap & Work Breakdown | Dependency-ordered milestones | RES-008,023–025,031,043–049 | CON-009; implementation gate |
| DOC-16 Traceability & Validation | Requirement/design/test/evidence links | RES-001–063 | All registers; EXT-01–03 |

Future Rust chapters should use a consistent structure: implementation problem → required concepts/patterns → isolated runnable example → invariants → correctness tests → benchmark → approved design links. Candidate topics include restricted producer/consumer handles, ownership transfer, immutable Arc snapshots, atomics, cache layout, traits/Send/Sync, bounded pools and safe retirement. Unsafe code remains a proposed technique, not a default requirement.

## 8. Output 5 — Decision Register

No entry below is an approved ADR. “Source-established” records research direction only. Recommendations are assessment proposals requiring the project's normal review process, not permission requests for completing this deliverable.

| ID | Status / proposed owner | Decision and preserved baseline | Assessment recommendation / alternatives | Linked records |
| --- | --- | --- | --- | --- |
| DEC-001 | Unresolved / Product | Financial TSDB vs edge database vs full trading engine | Define financial data/analytics boundary; keep execution capabilities separately scoped | RES-006–009,021,024; GAP-001; CON-001 |
| DEC-002 | Unresolved / Product | MVP feeds/features and staged delivery | Select first venue/feed and precise capability set; reconcile Stage 3 features with MVP | RES-010–013,025,043; CON-009 |
| DEC-003 | Proposed / Systems | Ring, double buffer and snapshot ownership | Compare restricted SPSC transfer, immutable batch registry and explicit buffer pool; retain unsafe proposals only with proof | RES-017,051,054,058,060–061; GAP-002/003; CON-004 |
| DEC-004 | Unresolved / Systems | Publication, flush and freshness policies | Separate visible-at, persisted-at and recyclable-at; choose thresholds and low-volume timer owner | RES-026,057,059; CON-003/007 |
| DEC-005 | Unresolved / Performance | Performance profile and measurement contract | Preserve MVP and EPYC profiles independently; qualify all figures until measured | RES-027,029,036–039,045,047–048; GAP-013; CON-002/008 |
| DEC-006 | Unresolved / Ingestion | Full ring and slow consumers | Evaluate block/backpressure, drop with accounting, disconnect/replay or bounded admission | RES-053; GAP-004; CON-005 |
| DEC-007 | Source-established direction; details unresolved / Domain | Nanosecond representation | Retain ns resolution; decide epoch/timezone/schema/sequence/overflow semantics separately | RES-040–042,051; GAP-006/014; CON-006/011 |
| DEC-008 | Unresolved / Storage | Durable acknowledgment and recovery | Evaluate volatile mode, WAL or source-replay durability; approve failure/commit model | RES-018,030; GAP-005 |
| DEC-009 | Proposed / Domain+Storage | Micro-sort, corrections and bitemporality | Define lateness bounds and knowledge versions; micro-sort alone cannot handle all corrections | RES-012,030,046,049; GAP-010 |
| DEC-010 | Source-established direction / Architecture | FDAP and single-node Rust MVP | Preserve foundation; pin dependencies and specify extension boundaries | RES-002,024; GAP-011 |
| DEC-011 | Proposed / Performance+Operations | EPYC reference, DPDK and core allocation | Correct population claim; compare ordinary sockets and bypass per adapter; measure locality and interference | RES-034–035; VAL-008/009/025; CON-010 |
| DEC-012 | Unresolved / Product | Open-core/proprietary component boundary | Select license, feature boundary and distribution policy independently of market claims | RES-008,022,031; CON-012 |
| DEC-013 | Proposed / Client Engineering | Flight/Python transport and API | Preserve Flight direction; define copy boundaries, types, nullability and client compatibility | RES-013–014,039,048; VAL-004 |
| DEC-014 | Unresolved / Domain | Financial schemas, SQL and order-book scope | Define precision, query semantics, instruments and venue reconstruction rules | RES-010–012,021,028,051; GAP-008/009 |

Decision closure requires an ADR with: source IDs; exact question; considered alternatives; rationale; product/technical approval record; validation evidence or explicit follow-up; affected documents; superseded records; effective version. Validating an implementation must not retroactively rewrite the original research claim.

## 9. Quantitative feasibility checks (reviewer-derived)

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

## 10. Phase 0 exit criteria and handoff

| Gate | Result | Evidence / condition |
| --- | --- | --- |
| Source identified and preserved | Complete | BASE-001 metadata, physical page count and SHA-256 |
| Every physical page classified | Complete | Page coverage ledger covers 1–42 |
| Significant source items extracted | Complete | RES-001–063, with consolidated repetitions |
| Five outputs populated | Complete | Outputs 1–5, not empty templates |
| Source and reviewer conclusions separated | Complete | Authority labels, validation scope, gaps and proposed decisions |
| Targets retained without invented results | Complete | Profile ledger and evidence limits |
| Documentation destinations assigned | Complete | DOC-01–16 mapping; all extraction IDs represented |
| Production engineering approval | Pending; outside Phase 0 | Critical gaps and ADRs require resolution |

Recommended next work, in dependency order:

1. Establish the Phase 1 MkDocs structure, document templates, reference convention and automated link/ID checks. Place the immutable source under research and this document under governance.
2. Resolve DEC-001/002 so the PRD can define an exact MVP and product boundary.
3. Specify correctness contracts: queue policy, ownership/snapshots, sequencing, durability and hot/cold visibility (DEC-003/004/006/007/008).
4. Pin the FDAP/Rust dependencies; create isolated safety/correctness prototypes before adapting research snippets.
5. Define workload-specific benchmark contracts for each retained profile; collect evidence before adopting performance guarantees.
6. Draft the PRD and architecture from extracted intent plus reviewed decisions. Preserve links to unresolved gaps rather than filling them with assumptions.

A future implementation task is ready only when it has a requirement, an approved design, explicit dependencies/ownership, measurable acceptance conditions and applicable verification evidence. Documentation scaffolding does not depend on every technical gap being closed; detailed unsafe implementation and release acceptance do.

## 11. Evidence register and maintenance

| ID | Evidence | Scope and limit | Access date |
| --- | --- | --- | --- |
| BASE-001 | HFT - Research Notes.pdf, identified above | Authority for original research intent; no complete bibliography or reproducible results | 2026-10-13 |
| EXT-01 | [Apache Arrow Flight RPC specification](https://arrow.apache.org/docs/format/Flight.html) | Flight uses gRPC/IPC; does not measure Chronos copy/latency costs | 2026-10-13 |
| EXT-02 | [Apache Arrow PrimitiveBuilder implementation](https://arrow.apache.org/rust/src/arrow_array/builder/primitive_builder.rs.html) | Inspected finish implementation transfers values; upstream live source, no PDF version pin | 2026-10-13 |
| EXT-03 | [AMD EPYC 9354 specifications](https://www.amd.com/en/products/processors/server/epyc/4th-generation-9004-and-8004-series/amd-epyc-9354.html) | Twelve memory channels; does not validate assembled server performance or locality | 2026-10-13 |

External sources qualify technical assertions; they do not replace BASE-001 as the original research record. Future evidence must retain versioned source snapshots or commits where feasible. Requirements derived from proposed corrections require their own rationale and approval trail.

On revision, append a change record, keep IDs stable, record changed source/evidence links and reassess affected downstream artifacts. Do not delete rejected research from historical extraction; mark its disposition and link the superseding decision.

| Version | Date | Change |
| --- | --- | --- |
| 1.0 | 2026-10-13 | Initial complete Phase 0 assessment; 63 extraction records, 28 validation records, 22 gaps, 12 conflicts, 16 document destinations and 14 decisions |
