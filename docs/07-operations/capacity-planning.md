# Capacity and overload planning

!!! info "Research-derived operations draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages) and the Product, Architecture and Engineering contracts linked below. Source proposals and derived operational procedures remain distinct. No implemented control, approved SLA, recovery guarantee or measured performance is asserted.


## Source profiles and derived arithmetic

The PDF's eight-core MVP and EPYC reference describe distinct unverified workloads (pp. 17, 21–23). Calculations below follow [Architecture topology](../02-architecture/deployment-topology.md) and the [performance ledger](../00-governance/performance-profiles.md); they are sizing inputs, not measured capacities or SLAs.

| Scenario | Derived raw payload | Operational qualification |
| --- | --- | --- |
| 2.5M ticks/s × 48 bytes | 120 MB/s; 10.368 TB/day | Excludes wire, Arrow metadata, catalog, indexes and storage encoding |
| Five-minute retention at that rate | 36 GB | Hot history is distinct from ingress queue and outstanding reader/result memory |
| 8M/s for 0.5 s, service at 2.5M/s | 2.75M excess ticks; 132 MB backlog | Assumes empty starting backlog and that service sustains its rate; burst repetition requires spare drain capacity |
| 2 × 3.84 TB striped drives | 7.68 TB nominal; about 17.8 hours raw payload | Before reserves/filesystem overhead; compression/write rate unmeasured; no redundancy |
| Example 1,024 × 4,096-row ring | 4,194,304 rows; about 4.19 s at 1M/s with no drain | Not five-minute hot history; three example value columns use about 80 MiB before other ownership |

After the burst, arrival equal to service leaves the excess backlog unchanged. Drain time is backlog divided by positive spare service rate; repeated bursts must fit the measured recovery interval. Use actual resident schema bytes rather than assuming the 48-byte payload equals memory consumption.

## Proposed resource budget

Account separately for network/decoder buffers, active builders, frozen/registry batches, reorder windows, delayed snapshot leases, query working memory, results/slow clients, persistence backlog and logs. Count shared allocations once in total memory while recording which owners delay reclamation. Retiring a registry entry or acknowledging a queue event does not free arrays still held by a reader (ENG-005/006).

Measure sustainable ingestion, publication, commit and egress simultaneously with the selected schema and query mix. Reserve disk space for in-progress files, catalog updates and compaction overlap; select retention and deletion with correction/as-known semantics. File-count and partition/row-group budgets are pending design, not implied by micro-Parquet (ENG-010–012).

## Proposed saturation policy

Under DEC-006 select bounded waiting/backpressure, explicit rejection/drop or disconnect/replay for each adapter. Define the observer, counters, affected range, retry safety and when queries are degraded. A live feed may not tolerate indefinite upstream blocking. Never overwrite live rows or allow unbounded buffering to hide pressure.

Set reviewed limits for query concurrency/work memory, result bytes, slow-client lifetime, retained snapshots and persistence backlog. Admission checks precede mutation. Source spin/drop/panic examples are not an approved operating policy (pp. 31–32; RES-053; VAL-012).

## Qualification and open choices

Record actual load/duration, burst repetition, publication age, latency distributions, losses, memory high-water marks, allocations/page faults, disk write/commit behavior and time-to-exhaustion. Compare sockets/bypass and locality under DEC-011; core pinning alone cannot establish isolation. Numeric capacity, retention reserves and alert thresholds require measured evidence under DEC-004/005/006/008/011.

Related: [Product benchmark targets](../01-product/benchmark-targets.md), [memory lifecycle](../03-engineering/active-memory-lifecycle.md), [persistence](../03-engineering/persistence-engine.md), [deployment](deployment.md), [observability](observability.md).
