# Engineering requirements

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


| Control | Value |
| --- | --- |
| Document | DOC-06 |
| Status | Proposed measurable engineering contracts; review pending |
| Companion | [Engineering design](engineering-design.md) (DOC-07) |

## Authority and boundary

The source establishes single-node Rust/FDAP direction and proposes concurrent ingestion, SQL, Parquet history and Flight/Python access (pp. 15–17, 26–28). Requirements below translate Product acceptance intent and Architecture contracts into observable engineering checks. All ENG IDs are local proposed requirements, not newly approved governance decisions. Source targets remain unverified; derived safety and correctness obligations require review.

## Requirement register

| ID / origin | Proposed engineering contract | Observable acceptance | Product / capability / governance trace |
| --- | --- | --- | --- |
| ENG-001 / Source direction | Run a single-node Rust daemon with compatible pinned FDAP dependencies | Clean offline build/start and a fixture-to-SQL workflow using recorded versions | F01; CAP-01/03/09; RES-002/024; DEC-010; GAP-011 |
| ENG-002 / Source + derived | Decode only declared adapter/event coverage; preserve gap, duplicate and malformed-input outcomes | Venue fixtures and reconnect/replay tests account for every admitted/rejected event | F02, N06; CAP-01; RES-025; DEC-002/006; GAP-004/018 |
| ENG-003 / Derived | Enforce exclusive mutation and capacity checks before append | Full/wrapped queues and concurrent caller attempts cannot overwrite live data or create mutable aliases | F03, N03/N08; CAP-02; RES-051/053/058; VAL-012/013/019 |
| ENG-004 / Source + derived | Publish initialized immutable batches at a declared row/timer boundary | Partial/empty/low-volume cases expose consistent column lengths; receipt-to-visible latency measured separately | F03, N02; CAP-02; RES-017/059; DEC-003/004 |
| ENG-005 / Derived | Acquire non-destructive query snapshots and retain backing storage through all readers | Repeated/concurrent scans return the declared snapshot; delayed reader survives wraparound | F03, N08; CAP-02; RES-054/060; VAL-015/020/021 |
| ENG-006 / Source + derived | Bound memory across ingestion, snapshots, query execution and persistence | Saturation applies declared policy with accounting; allocation, deallocation and page-fault audit covers turnover | N03/N06; RES-020/044/052; DEC-003/006; GAP-012 |
| ENG-007 / Source + derived | Preserve time, identity, sequence, price and quantity semantics across tiers/clients | Boundary, equal-time, numeric precision, null and schema round-trip fixtures pass | F12; CAP-08; RES-040–042; DEC-007/014; VAL-010/011/018 |
| ENG-008 / Source + derived | Support a declared SQL subset and correct projection/filter/limit behavior | Oracle comparisons include reordered projections, empty input and unsupported pushdown | F04; CAP-03; RES-028/054; VAL-016/017/026 |
| ENG-009 / Source + derived | Implement ratified ASOF and bucket/OHLCV semantics | Independent financial fixtures cover ties, boundaries, keys, nulls and corrections | F05/F06; CAP-03; RES-010/011; DEC-014; GAP-009 |
| ENG-010 / Source + derived | Persist Parquet under a declared commit and recovery model | Fault injection/restart yields exactly the promised committed history; incomplete files remain ineligible | F07, N07/N09; CAP-04; RES-018; DEC-008; VAL-024 |
| ENG-011 / Derived | Select each eligible logical event/version exactly once across hot/cold handoff | Concurrent flush/eviction queries match a fixed reference dataset without omissions/duplicate aggregates | F08, N08; CAP-05; RES-019/047; GAP-007; VAL-023 |
| ENG-012 / Source + derived | Preserve late/corrected versions and explicit knowledge-time visibility | Current and as-known queries agree with correction oracle before/after commits and compaction | F09/F10; CAP-06; RES-012/030/049; DEC-009 |
| ENG-013 / Source + derived | Stream typed results through a declared Flight/Python contract | Supported clients preserve types/nulls/ns timestamps; cancellation and mid-stream failure release resources | F11; CAP-07; RES-013/048; DEC-013; VAL-004 |
| ENG-014 / Source + derived | Qualify each adopted performance target by workload and endpoint | Reproducible run records latency distribution, losses, resources, freshness and durability mode | N01/N02; RES-027/029/036–039; DEC-005/011; GAP-013 |
| ENG-015 / Source + derived | Support private/offline runtime with explicit access boundaries | Approved package runs local fixtures offline; feed/client/admin permissions and failures are exercised | N04/N05; CAP-09; RES-015; GAP-015/018 |

F/N shorthand means PRD-F/PRD-N; capabilities are from [Product architecture](../01-product/product-architecture.md). Candidate book reconstruction F13/CAP-10 remains separately scoped and needs venue-specific oracle acceptance (VAL-027).

## State and performance contracts

Record accepted, published/query-visible, durable and recyclable states separately. Accepted ingest is not a durable acknowledgment. Publication age, SQL execution latency, query cadence and cold-flush period are independent measurements.

Retain PERF-MVP-ING (up to 1M ticks/s on eight cores) and PERF-MVP-Q (hot aggregate P99 < 1 ms) as source targets (p. 17). The EPYC reference profiles, 8M/s burst, hybrid < 50 ms and remote 50M-row < 1.5 s scenarios are distinct workloads, not a combined SLA. Use the [performance ledger](../00-governance/performance-profiles.md) and [validation plan](validation-and-traceability.md) before ratifying numeric acceptance.

## Approval inputs

Review DEC-001/002 scope, DEC-003 ownership, DEC-004 freshness, DEC-005 performance, DEC-006 overload, DEC-007 clocks, DEC-008 durability, DEC-009 corrections, DEC-011 hardware, DEC-013 client and DEC-014 domain contracts. Named approvers, evidence and dates remain pending. Proposed requirements become release gates only through recorded review.
