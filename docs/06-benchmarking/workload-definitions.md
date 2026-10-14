# Workload definitions

!!! info "Source targets and proposed fixtures · 2026-10-14"
    Targets come from the [HFT research notes](../assets/research/HFT-Research-Notes.pdf), physical pp. 17, 21–23 and 26–28, and the [Product benchmark targets](../01-product/benchmark-targets.md). All are untested. Fixture details below are proposed refinements, not approved release contracts.

## Source-derived profile ledger

| Existing profile | Target / source | Workload contract to complete |
| --- | --- | --- |
| PERF-MVP-ING | Up to 1M ticks/s continuously on standard 8-core bare metal; p. 17 | Adapter/schema, duration, sustained loss accounting, concurrent queries and durability |
| PERF-MVP-Q | Hot-window aggregate P99 < 1 ms; p. 17 | SQL, window/rows, instruments, concurrency, cache and timing boundary |
| PERF-REF-ING | 2.5M ticks/s, average 48-byte L2 payload on EPYC reference; pp. 21–22 | Wire/resident bytes, decode cost, simultaneous persistence/query load |
| PERF-REF-BURST | 8M ticks/s for up to 500 ms; p. 23 | Initial occupancy, burst repetitions/spacing, service rate, drops and drain time |
| PERF-REF-Q | Time-bucket/VWAP P95 < 180 µs; P99 < 850 µs under simultaneous load; p. 23 | Exact window, output size and engine/client endpoints |
| PERF-REF-EGRESS | Approximately 4.2 GB/s, loopback/100GbE wording; p. 23 | Separate local/remote runs, actual bytes and full client completion |
| PERF-STAGE-1 | 1M ticks/s with SQL every 100 ms; pp. 26–27 | AVG/COUNT oracle, cadence versus execution latency; lifetime safety evidence |
| PERF-STAGE-2 | Correct hybrid aggregate < 50 ms; p. 27 | One-hour example: 55 minutes cold + five hot; pin actual rows, cold bytes and cache state |
| PERF-STAGE-3 | 50M rows into remote Polars < 1.5 s with ASOF SQL; pp. 27–28 | Input versus output rows, join semantics/width, network and dataframe endpoint |
| PERF-DX / PERF-SIGNAL | 20× notebook loading (p. 17) / often < 100 ns signaling (p. 42) | Matched independent baseline / isolated signaling harness; neither is an end-to-end database gate |

## Additional product target profiles

The [product target document](../01-product/benchmark-targets.md) supplements the ledger; it does not replace it. The labels below are local workload labels, not new approved governance IDs.

| Local label | User-proposed target | Distinction to retain |
| --- | --- | --- |
| Product sustained/burst | 1–2M ticks/s per node; 8M ticks/s for up to 1,000 ms; zero eligible-event drops; 48–64 bytes/tick | One-second burst differs from PERF-REF-BURST's half-second window |
| Product query tails | P50 < 50 µs, P99 < 150 µs, P99.9 < 850 µs | Simple lookup/bounded single-instrument VWAP; continuous 1M ticks/s, persistence/maintenance for tail evaluation |
| Product local dataframe | 50M rows < 1.5 s over loopback | Remote Stage 3 remains separate; output width determines byte rate |
| Product storage | 1B × 64-byte ticks: < 8.5 GB Parquet footprint; 8×–10× compression aspiration | 64 GB raw decimal size; footprint gate implies > 7.53×, not automatically 8×–10× |

## Dataset and query fixtures

Declare venue/protocol version and framing, symbol population/skew, price/size representation, nullability, timestamp unit/timezone, sequence and correction identifiers. Keep the three-column prototype (timestamp/price/size, 20 logical bytes) separate from 48-byte L2 and 48–64-byte product payloads. Report encoded network, logical and resident sizes independently (PRD-F02/F12).

Use fixed seeds and retain generator configuration/input hashes. Cover uniform and hot-symbol skew, quiet periods, realistic sustained traffic and repeated bursts. Run bounded aggregate/lookup, projection/filter/limit, VWAP, time-bucket/OHLCV and ASOF fixtures only against their declared semantics; preserve exact SQL and an independent expected result. L2/L3 reconstruction is conditional on scope approval rather than inferred from a `live_order_book` table name.

| Fixture | Perturbation | Required observation |
| --- | --- | --- |
| Low volume | Partial batches, no traffic, timer expiry | Receipt-to-visible delay; writer-owned flush behavior |
| Saturation | Burst/full ring; slow query, slow disk or stalled consumer | Explicit admission outcome, bounded memory and recovery between bursts |
| Sequencing | Equal timestamps, duplicates, gaps, reconnect and within/beyond-window lateness | Declared ordering/status and no silent loss |
| Hybrid handoff | Queries before/during/after commit and compaction | Each eligible event/version contributes exactly once |
| Corrections | Late correction before/after tier transition | Current versus as-known answers under DEC-009/014 |
| Client | Loopback/remote, Polars/Pandas, cancellation and partial errors | Complete rows, type fidelity, conversion cost and reference release |
| Recovery | Crash/disk-full/write failure during persistence | Outcomes consistent with approved acknowledgment/recovery mode |

## Capacity and clock qualifications

With 2.5M/s service, an 8M/s 500 ms burst adds `(8M − 2.5M) × 0.5 × 48 = 132 MB` excess logical backlog. For the product one-second burst, a hypothetical 2M/s service rate and 64-byte events add 384 MB; these are derived illustrations, not measured buffer requirements. Include resident overhead, outstanding snapshots and repeated-burst drain.

The example 1,024 × 4,096 ring holds 4,194,304 rows: about 4.19 seconds at 1M/s without drain, not five minutes of history. Hot retention is a separate budget. Resolve 5/10-second cold flush and 500 µs/1,000-row versus 4,096-row publication through DEC-004; record both clocks independently (CON-003/007).

Related: [methodology](benchmark-methodology.md), [deployment topology](../02-architecture/deployment-topology.md), [query/storage integration](../02-architecture/query-and-storage.md), [Scope & MVP](../01-product/scope-and-mvp.md).
