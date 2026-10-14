# MVP delivery plan

!!! info "Proposed MVP · 2026-10-14"
    This page translates the [Product scope and MVP definition](../01-product/scope-and-mvp.md) into delivery and release gates. Product scope remains authoritative; DEC-001/002 are unresolved. No implementation or benchmark pass is asserted.

## Complete proposed MVP

The source MVP (BASE-001 pp. 16–17) combines single-node Rust/FDAP, selected feeds, concurrent recent-data SQL, Parquet history and unified queries, financial operators, late/corrected events with as-known history, nanosecond representation and Flight/Python access. Its complete delivery spans all three source stages (pp. 26–28). Source FR-3 bitemporality remains included despite its later delivery placement (CON-009).

| Capability / PRD ID | Delivery milestone | Required acceptance evidence |
| --- | --- | --- |
| Single-node Rust/FDAP — F01 | RM-02/04 | Pinned supported platform and compiling runtime |
| Feed adapters — F02 | RM-04 | Approved venue/version matrix; parse, gap, reconnect and replay fixtures |
| Concurrent recent-data SQL — F03/F04 | RM-04 | Non-destructive reads, snapshot lifetime, filter/projection/limit correctness under ingestion |
| ASOF, TIME_BUCKET and OHLCV — F05/F06 | RM-06 | Independent oracles for ties, boundaries, nulls and ordering under approved semantics |
| Parquet history and unified reads — F07/F08 | RM-05 | Commit/restart and hot/cold handoff correctness under the selected failure model |
| Late events, corrections and as-known history — F09/F10 | RM-05/06 | Approved event/knowledge versions; late/corrected fixtures and historical query agreement |
| Flight/Python/Polars/Pandas — F11 | RM-06 | Supported client matrix; types, precision, cancellation and partial-failure evidence |
| Nanosecond representation — F12 | RM-02/04 | Epoch/timezone/sequence/overflow contract and round-trip fixtures |
| Private/offline runtime — N04 | RM-07 | Install/run artifact inventory, access configuration and deployment verification; venue connectivity separately documented |

The adapter family includes Binance/Coinbase WebSockets and ITCH structures in the proposed Product scope. DEC-002 must specify the exact release coverage, including live versus mock ITCH; a fixture-only demonstration cannot imply a supported live adapter.

## Incremental demonstrations

| Slice | User-visible increment | Source target, pending workload approval | Release boundary |
| --- | --- | --- | --- |
| A — Recent data | Capture selected events and query live aggregates concurrently | Up to 1M ticks/s; hot P99 < 1 ms on the MVP profile; queries every 100 ms | Demonstrates recent-data path; full MVP features remain outstanding |
| B — Historical continuity | Query recent and persisted data with a consistent tier transition | Correct hybrid aggregate < 50 ms | Requires commit/recovery and late/correction plumbing evidence |
| C — Financial workflow | Query ASOF/bars/as-known history and consume results in Python | Remote 50M-row Polars scenario < 1.5 s | Completes capability delivery; operational release gates still apply |

These figures are source targets, not guarantees or results. Use the [benchmark acceptance criteria](../06-benchmarking/acceptance-criteria.md) for adopted workloads and the [registry](milestone-register.md) for milestone evidence.

## Scope boundaries

Full order-book reconstruction (F13), object-storage cold paths, fleet management, active-passive HA and commercial support are extensions or optional proposals requiring separate scope decisions. Edge/IoT features are historical alternatives. Trading strategies, OMS, pre-trade risk and order routing are proposed external consumers. Distributed query, sharding and consensus are not established MVP requirements. Preserve these dispositions in Product scope; this delivery page does not approve exclusions.

## Release checklist

- [ ] DEC-001/002 records approve the product boundary, feeds and complete capability matrix; any reduced launch scope has explicit requirement dispositions.
- [ ] Required schema, financial, ownership, visibility, overload, durability and client contracts are reviewed and versioned.
- [ ] Required safety and correctness evidence covers concurrent ingestion/query, financial operators and correction/as-known states.
- [ ] Recovery and hot/cold transitions pass fixtures under the declared acknowledgment/failure model without silently missing or double-counting eligible events.
- [ ] Every adopted performance profile has reproducible, workload-qualified results, including errors/losses, memory/backlog, freshness and durability context.
- [ ] Supported dataframe clients and private/offline deployment pass the documented compatibility and packaging checks.
- [ ] Required security, capacity, retention, operational and recovery checks pass for the approved deployment scope.
- [ ] RM-07 records named release approval, artifact versions, evidence links and any accepted residual risks.

All checklist items are pending. Documentation and a successful site build establish navigability, not runtime or release acceptance.
