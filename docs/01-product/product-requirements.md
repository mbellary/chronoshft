# ChronosHFT — Product Requirements Document

| Control | Value |
| --- | --- |
| Document | DOC-02 / CHRONOS-P2-PRD-001 |
| Version / date | 1.0 / 2026-10-13 |
| Status | Complete product-definition draft; product approval and performance validation pending |
| Authority | Derived from BASE-001; not an approved replacement for the research |
| Related documents | [Product Architecture](product-architecture.md); [Scope and MVP](scope-and-mvp.md); [Phase 2 handoff](phase-2-definition.md) |

## 1. Baseline and rules

BASE-001 is **HFT - Research Notes.pdf**, 42 physical PDF pages, SHA-256 `18b5ed26b6f4dfd8a83c452abf7cb81946f338078e66c85c71f48343631f37a2`. Its PRD is marked Draft / Vision Realisation (p. 14). References below use physical PDF pages. Phase 0, CHRONOS-P0-001 v1.0, supplies stable RES, GAP, CON and DEC identifiers. The supplied draft was authored before repository integration; it is now published within the existing Phase 1 documentation layout.

The PDF remains unchanged. Source requirements, source proposals, unverified targets, and reviewer-derived acceptance conditions are distinguished. Inclusion here does not close Phase 0 decisions or approve prototype code. No benchmark, customer validation or implementation result is claimed. No requirements from adjacent projects are imported.

**Authority vocabulary:** S = source-stated requirement/direction; P = source proposal; T = unverified target; D = reviewer-derived candidate requirement or acceptance refinement. All requirements below await product approval. Priority means recommended product priority, not an existing approved commitment. “Core” means proposed complete MVP; delivery slices are not independently the complete MVP.

## 2. Product vision and outcomes

ChronosHFT aims to reduce friction between financial market-data capture and quantitative analysis through a developer-first real-time financial time-series database. The source couples a Rust single-node engine with Flight, DataFusion, Arrow and Parquet (FDAP), financial SQL, and Python workflows. It targets mid-market hedge funds, proprietary trading desks and digital-asset market makers (pp. 14–18; RES-002/009/023/024).

The working product boundary is market-data storage and analytics. A complete strategy execution, order-management, pre-trade risk and exchange order-routing product is not established by the baseline. This is a proposed interpretation pending DEC-001, not a silent narrowing of approved scope.

| Outcome ID | User outcome | Success evidence | Source / qualification |
| --- | --- | --- | --- |
| OUT-01 | Capture live financial events while analysts query them | Workload-linked ingestion and concurrent-query evidence | pp. 16–17, 26–27; RES-027/029/045; unverified targets |
| OUT-02 | Use SQL and notebooks without a separate CSV pipeline | Reproducible query-to-dataframe workflow, measured type fidelity and transfer costs | pp. 6–7, 17, 27–28; RES-013/028; no absolute network zero-copy promise |
| OUT-03 | Analyze recent and historical data through one interface | Correct cross-tier query compared with a reference dataset | pp. 15–16, 27; RES-019/047 |
| OUT-04 | Reproduce historical state of knowledge after corrections | Before/after correction fixtures and as-known query evidence | p. 17, 28; RES-012/049 |
| OUT-05 | Retain control of deployment and data | Offline deployment test and documented Arrow/Parquet export compatibility | pp. 7, 15, 17; RES-015/023 |
| OUT-06 | Reduce infrastructure burden and time to research | Customer workflow study; setup time and time-to-first-useful-query | pp. 14–15; RES-023; days-to-minutes aspiration has no validated baseline |

## 3. Personas and use cases

Personas are research-derived hypotheses, not completed customer interviews (GAP-019).

| Persona | Need | Product workflow | Limit |
| --- | --- | --- | --- |
| Quantitative researcher | Align trades/quotes, derive bars and inspect corrected history | SQL → Flight → Polars/Pandas/Jupyter | Strategy development remains external |
| Trading-data engineer | Capture feeds and manage live/historical availability | Configure supported adapter → ingest → inspect progress and history | Venue contracts and feed entitlements pending |
| Prop-desk / fund evaluator | Private deployment with predictable performance | Run representative pilot and compare correctness/latency | No profitability or competitor superiority guarantee |
| Platform operator | Know whether capture is healthy and recover failures | Inspect lag/errors/resource state → apply documented recovery | Operational persona derived from private deployment; controls proposed |

| Use case | Trigger and outcome | Requirement links |
| --- | --- | --- |
| UC-01 Live capture | Supported feed emits events; user can query published events while capture continues | PRD-F01/F02/F03/N01/N02 |
| UC-02 Trade/quote analysis | Analyst requests a preceding quote for each trade; matching follows declared semantics | PRD-F04/F05 |
| UC-03 OHLCV | Analyst selects instrument and interval; bars have documented boundaries and empty-window rules | PRD-F06 |
| UC-04 Historical analysis | Query spans memory and persisted history; each eligible event contributes once | PRD-F07/F08 |
| UC-05 As-known research | Correction arrives; analyst can distinguish current truth from a previous knowledge state | PRD-F09/F10 |
| UC-06 Notebook research | Client submits SQL and receives typed results suitable for a dataframe | PRD-F11 |
| UC-07 Private operation | Operator installs approved packages in an isolated environment and runs capture/query | PRD-N04/N05 |

## 4. Functional requirement register

Acceptance entries are **proposed observable checks**, not test results. Detailed protocols, algorithms and fixtures belong to subsequent engineering/domain documents.

| ID / priority / authority | Product requirement | Proposed acceptance and unresolved detail | Baseline trace |
| --- | --- | --- | --- |
| PRD-F01 / Core / S | Deliver a single-node Rust database binary with embedded DataFusion | Start one node, configure capture and execute SQL; supported OS/package matrix pending | p. 16, 26; RES-024; DEC-010 |
| PRD-F02 / Core / P | Provide digital-asset WebSocket adapters and standard ITCH-structure ingestion | Replay adapter-specific fixtures with declared mapping; Binance/Coinbase appear in source; first venue, protocol version and live vs mock ITCH coverage pending | p. 16, 26; RES-025; GAP-018; DEC-002 |
| PRD-F03 / Core / P | Make recent market events queryable while ingestion continues | Concurrent capture/query yields expected events; visibility boundary and freshness target pending | pp. 16, 26–27; RES-016/017/045; GAP-003 |
| PRD-F04 / Core / S | Provide SQL with financial time-series extensions | Publish supported syntax/type/function matrix and execute golden queries; ANSI SQL is source intent, not a claim of universal conformance | p. 17; source FR-2; RES-028 |
| PRD-F05 / Core / P | Support ASOF matching of asynchronous trade and quote streams | Reference fixture checks preceding-match behavior; ties, tolerance, instrument keys and unmatched rows pending | pp. 6, 16, 28; RES-010; GAP-009 |
| PRD-F06 / Core / P | Support time bucketing and OHLCV generation | Compare bars with independent oracle; interval alignment, timezone/calendar, empty bars and tick windows pending | pp. 6, 17, 28; RES-011; GAP-009 |
| PRD-F07 / Core / P | Persist historical events in Parquet through background processing | Persisted files can be read with documented schema; flush alternatives retained; durable acknowledgment/recovery pending | pp. 9, 16, 27; RES-018/026; GAP-005 |
| PRD-F08 / Core / P | Query hot and cold data through a unified interface | Cross-boundary aggregate matches source dataset with no omission/double counting; proposed correctness refinement D | pp. 16, 27; RES-019/047; GAP-007 |
| PRD-F09 / Core / S | Handle late data and corrections without rewriting the entire historical dataset | Apply late/corrected fixtures and verify declared views; lateness limits and rewrite scope pending | p. 17, 27–28; RES-030/046/049; GAP-010 |
| PRD-F10 / Core / S | Provide a bitemporal state-of-knowledge query option | Query before/after knowledge boundary returns declared historical versions; clock/version model pending | p. 17, 28; source FR-3; RES-012/049; DEC-009 |
| PRD-F11 / Core / P | Provide Python query access using Arrow Flight and dataframe integration | End-to-end typed results in documented supported Polars/Pandas versions; nulls/timestamps/types checked; conversion/copy costs reported | p. 17, 27–28; RES-013; DEC-013 |
| PRD-F12 / Core / S | Preserve nanosecond timestamp representation across the data path | Round-trip supported values without precision truncation; sequence, clock, timezone and overflow semantics pending | pp. 23–26; RES-040; DEC-007 |
| PRD-F13 / Candidate extension / P | Support historical L1/L2/L3 order-book reconstruction | Requires separately approved venue coverage and reconstruction rules; not implied by ingesting ITCH or naming a table live_order_book | p. 10; RES-021; GAP-006; DEC-014 |

## 5. Nonfunctional and correctness requirements

| ID / priority / authority | Requirement | Acceptance boundary | Trace |
| --- | --- | --- | --- |
| PRD-N01 / Core / S+T | Target continuous ingestion up to 1M ticks/s on standard 8-core bare metal | PERF-MVP-ING; exact machine, schema, adapter, duration, loss and concurrent load unresolved | p. 17; source FR-1; RES-027; GAP-013 |
| PRD-N02 / Core / S+T | Target P99 below 1 ms for live in-memory window aggregates | PERF-MVP-Q; query/data/window/concurrency/timer endpoints unresolved; not ingest-to-query freshness | p. 17; source NFR-1; RES-029 |
| PRD-N03 / Core / S; refinement D | Avoid GC pauses; control memory behavior in the performance-critical path | Source demands deterministic memory; specify bounded budgets, allocation audit region and saturation behavior before making a guarantee | p. 17, 26; source NFR-2; RES-020/044; GAP-012 |
| PRD-N04 / Core / S | Support self-hosted/private VPC/co-location operation without runtime external internet dependency | Install from approved offline bundle; exercise supported local fixtures/clients without external services; live Internet venue feeds still require their connectivity | pp. 7, 17; source NFR-3; RES-015 |
| PRD-N05 / Core candidate / D | Make deployment, access and feed permissions explicit | Threat model and adapter entitlement boundary; authentication, encryption and administrative controls pending, not inherited from VPC placement | GAP-015/018; derived from PRD-N04/F02 |
| PRD-N06 / Core candidate / D | Report gaps, duplicates and overload outcomes rather than silently presenting incomplete data as complete | Fault fixtures produce declared status/loss accounting; policy choice pending | GAP-004/006; RES-025/053 |
| PRD-N07 / Core candidate / D | Declare visibility, durability and recovery guarantees separately | Failure test matches approved acknowledgment model; no zero-loss claim until DEC-008 resolves | GAP-005/007; RES-018/030 |
| PRD-N08 / Core candidate / D | Preserve query correctness under concurrent publication and persistence | Repeated/concurrent reads and tier-handoff fixtures pass declared consistency model | GAP-003/007; RES-019/054 |
| PRD-N09 / Core / S; refinement D | Preserve open-format access to user data | Export/read compatibility matrix; no unproven zero-conversion guarantee across all tools | p. 15; RES-023/004 |

Derived candidates make missing product contracts reviewable; they do not automatically become approved additions. Price/size precision, instrument identity, sequence provenance and clock quality remain GAP-006/008/014 and must be resolved before financial correctness is accepted.

## 6. Performance profile ledger

All figures are **unverified source targets**. Benchmark qualification and measured evidence are future deliverables. No profile is inferred from another.

| Profile | Target and source conditions | PDF / RES | Qualification needed |
| --- | --- | --- | --- |
| PERF-MVP-ING | Up to 1M ticks/s continuously; standard 8-core bare metal | p. 17 / RES-027 | Schema, feed cost, duration, durability/loss, concurrent query load |
| PERF-MVP-Q | P99 < 1 ms; hot window aggregates | p. 17 / RES-029 | Windows, rows, query mix, concurrency and client/server timing |
| PERF-REF-ING | 2.5M ticks/s sustained; average 48-byte L2 tick; EPYC reference | pp. 21–22 / RES-034/036 | Wire vs resident bytes, run length, end-to-end accounting |
| PERF-REF-BURST | Up to 8M ticks/s for up to 500 ms | p. 23 / RES-037 | Queue baseline/headroom, drain, repetition and loss policy |
| PERF-REF-Q | P95 < 180 µs; P99 < 850 µs; time-bucket/VWAP under simultaneous load | p. 23 / RES-038 | Workload and timing endpoints |
| PERF-REF-EGRESS | Approximately 4.2 GB/s; loopback/100GbE wording | p. 23 / RES-039 | Separate local/remote runs and actual client completion |
| PERF-STAGE-1 | 1M ticks/s with query every 100 ms | pp. 26–27 / RES-045 | Query cadence is not query latency or freshness |
| PERF-STAGE-2 | Cross-tier aggregate < 50 ms | p. 27 / RES-047 | Dataset, cold bytes, cache state and correctness oracle |
| PERF-STAGE-3 | 50M rows into remote Polars < 1.5 s, SQL containing ASOF | pp. 27–28 / RES-048 | Input vs output rows, width, result size, network and dataframe endpoint |
| PERF-DX | Notebook loading 20× PostgreSQL/CSV | p. 17 / RES-032 | Matched independent baseline; marketing hypothesis only |

Flush every 5 s or 500k ticks (p. 9) and every 10 s or 500k ticks (pp. 16, 27) remain CON-003. Publication at 500 µs/1k rows and the 4,096-row example remain CON-007. None establishes a product freshness SLA. The EPYC/DPDK blueprint is a reference proposal, not mandatory minimum hardware or proof of any target.

## 7. Product positioning and adoption

The source proposes SQL/Python familiarity, concurrent capture and research, open formats and private deployment as differentiation (pp. 14–20). Claims about kdb+, ClickHouse and DuckDB are research assertions; no new competitive conclusions are adopted here. Validate comparisons through matched workloads before publishing superiority claims.

The go-to-market proposal is open-source single-node adoption through quant communities and educational notebooks, followed by paid fleet management, active-passive HA and support. Throughput/resource pricing and unlimited developer seats are proposals (pp. 17–18; RES-031). License, open/proprietary boundaries, pricing and SLA obligations remain DEC-012/GAP-019/021.

Recommended evaluation evidence, not numeric launch commitments: successful representative pilot; correct finance fixtures; time to installation and first query; reliable offline operation; measured performance/cost; repeat use by intended personas. No conversion, revenue or customer-count target is invented.

## 8. Product risks and open decisions

| Risk / decision | Impact | Required disposition |
| --- | --- | --- |
| DEC-001 / GAP-001 | Product boundary can expand into execution or edge/IoT | Approve exact scope and exclusions |
| DEC-002 / CON-009 | MVP includes features delivered in later roadmap stages | Approve complete MVP versus delivery slices and first adapter |
| DEC-004 / CON-003/007 | Flush/publish timings imply inconsistent freshness | Define visibility target and independent persistence policy |
| DEC-005 / GAP-013 | Unqualified performance claims become release promises | Approve workload-specific acceptance contracts |
| DEC-006/008 | Finite memory and process failures expose unknown loss behavior | Select overload and durability/recovery modes |
| DEC-007/009/014 | Time/financial/correction semantics can change analysis results | Approve observable domain rules |
| DEC-012/013 | Commercial/API expectations exceed evidence | Define license, feature tiers and compatibility boundary |
| GAP-002/003/011/012 | Prototype code is mistaken for safe implementation | Resolve through engineering design and validation |

## 9. Acceptance and authority

Product definition is drafted when every requirement has a source or explicit derived rationale, scope disposition and observable verification intent. Product baseline approval additionally requires recorded scope/MVP decisions and product-authorized requirements. Release acceptance requires correctness, reliability and workload-qualified benchmark evidence; the source nine-month schedule is not an approved delivery commitment.

No decision is marked approved by this document. Proposed accountable roles are Product for scope/value, Domain for semantics, Architecture for capability boundaries, Engineering for contracts, and Validation for evidence. Named owners, dates and approval records remain unassigned.
