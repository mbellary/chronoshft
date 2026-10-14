# Query and storage integration

!!! info "Research-derived integration draft · 2026-10-14"
    Source: [unified HFT research notes](../assets/research/HFT-Research-Notes.pdf), pp. 6–17, 24–28 and §3.2 (pp. 33–36). Snapshot/catalog mechanisms below are proposed corrections to the prototypes.

## Custom active-memory TableProvider

The source's `HftActiveTableProvider` exposes the Arrow schema and a DataFusion `scan()`, polling frozen batches into a memory execution plan. It is registered as `live_order_book` in a `SessionContext`; a demonstration periodically runs `AVG(price)` and `SUM(size)` over a price filter (pp. 33–35; RES-054/055). The name does not mean the example reconstructs an order book.

Polling the rotator drains its shared queue. Later or concurrent scans can therefore see different/disjoint data. **Proposed provider contract:** capture references from a retained immutable registry at a defined snapshot cutoff, without advancing ingestion consumption or evicting another reader's data (VAL-015).

The sample projects each batch and also passes a projection with the original schema into the execution plan. Pin the DataFusion API and define projection once with matching output schema; test reordered and sparse column selection. Filter pushdown and limits are not implemented by the sample, so the planner must retain responsibility for unsupported operations rather than receiving false pushdown guarantees (VAL-016/017).

## Hybrid catalog and routing

The source describes routing a one-hour window across 55 minutes of Parquet and five minutes of Arrow memory (pp. 12, 16, 27; RES-019/047). The split is illustrative. The catalog must represent eligible sources and their temporal/version coverage; hot retention is a policy, not a fixed ring length.

**Proposed plan:** acquire one consistent memory/catalog view, prune eligible files/batches using supported metadata, scan both tiers, apply required ordering/correction semantics and execute the query over their logical union. A plain concatenation is insufficient if the same events exist in both tiers. Commit boundaries and version identities must prevent omissions and duplicate aggregates during flushing (DEC-008/009; VAL-023).

Partitioning, file names, min/max statistics, compaction and schema evolution need separate decisions. The PDF asks for these details but does not supply a concrete storage-layout algorithm. Micro-Parquet writes preserve columnar interoperability; they do not alone define durable acknowledgment or restart consistency (VAL-024).

## Financial execution semantics

| Capability from research | Source | Contract still needed |
| --- | --- | --- |
| ASOF matching | pp. 6, 16–17, 28 | Join direction, keys, tie-breaking, tolerance, ordering and null behavior |
| TIME_BUCKET / OHLCV | pp. 17, 28 | Bucket origin/timezone, interval boundaries, late data and open/close ordering |
| VWAP and window analytics | pp. 9, 23 | Price/quantity precision, volume definition and window semantics |
| Bitemporal/as-known lookup | pp. 6–7, 28 | Event time versus knowledge time, version identity and correction visibility |
| L1/L2/L3 reconstruction | p. 10 | Venue-specific add/modify/delete semantics, gaps and replay; candidate extension |

These capabilities may require custom DataFusion functions, planner rules or operators; their implementation and SQL subset are unresolved (DEC-014; VAL-026/027). Bounded micro-sorting in Stage 2 cannot replace a full correction/version model.

## Timestamp and client boundaries

Preserve nanosecond semantics through Arrow, Parquet and Flight. The proposed Arrow Timestamp and Parquet NANOS logical type need an agreed UTC/timezone convention, schema and overflow handling. Preserve sequence information when equal timestamps occur (pp. 24–25; VAL-010/011/018).

Arrow Flight is the source's Python result boundary, with Polars/Pandas/Jupyter workflows in Stage 3 (pp. 17, 27–28). Specify the query request/RPC contract, cancellation, errors, authentication and type compatibility before promising a client API. No Flight SQL or subscription endpoint is assumed. Arrow-oriented transport can reduce format conversion, but remote transport is not shared address-space memory and does not guarantee no serialization or copying (DEC-013; VAL-004).

## Integration evidence required

Verify repeated and concurrent scans, projections, supported filters, snapshot cutoff, hot/cold handoff under ongoing ingest, corrections and timestamp round trips. Inject incomplete writes and restart failures once a commit protocol is selected. Measure hot-only, cold-only, hybrid and remote result workloads separately; the source's latency/throughput figures are targets (see [performance profiles](../00-governance/performance-profiles.md)).

Related: [data flow](data-flow.md), [component contracts](component-contracts.md), [technical validation](../00-governance/technical-validation.md), [financial query semantics](../05-financial-domain/financial-query-semantics.md).
