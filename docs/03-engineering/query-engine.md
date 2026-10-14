# Query engine

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


## Research implementation and correction

The PDF `HftActiveTableProvider` builds a memory execution plan by polling batches from the rotator, then registers `live_order_book` and runs aggregate SQL (pp. 33–36; RES-054/055). Polling advances the shared queue cursor, so repeated/concurrent queries may see disjoint data (VAL-015). The table name does not establish order-book reconstruction. Proposed implementation follows [Architecture query/storage](../02-architecture/query-and-storage.md) and ENG-005/008/009/011/012.

## Proposed snapshot scan algorithm

1. Capture a declared snapshot cutoff and a consistent retained-memory/committed-catalog view.
2. Acquire references to eligible immutable batches/files without consuming the ingestion queue.
3. Select logical event/version coverage once across hot/cold sources, applying the approved correction/knowledge policy.
4. Construct scan partitions with schema-compatible inputs and declared ordering. Prune using valid metadata without excluding eligible late versions.
5. Apply projection once, then only supported filter/limit pushdown; leave residual operations to the planner.
6. Execute financial operators and stream typed results. Release references on completion, cancellation or failure.

Snapshot/catalog atomicity and membership semantics are proposals under DEC-003/009. A snapshot may omit still-mutable rows by design; expose its boundary and measure publication freshness separately from execution latency.

## Provider and plan correctness

| Concern | Required engineering behavior |
| --- | --- |
| Dependency API | Pin DataFusion/Arrow versions and compile complete provider implementation; source missing helpers/API uncertainty remain VAL-017 |
| Projection | Keep projected batches, output schema and plan indices consistent; avoid source double application (VAL-016) |
| Filters | Declare only supported pushdown; preserve residual predicate evaluation |
| Limits | Push only when semantically valid across partitions, ordering and financial operators |
| Empty scans | Return valid typed empty results; do not invent a row or panic |
| Ordering | Declare partition/order guarantees; apply sorting where financial semantics require it |
| Schema evolution | Resolve compatible schema versions explicitly, including nullability and numeric/time precision |
| Resource limits | Bound query scratch/sorts/concurrency and snapshot lifetime under ENG-006 |

No current upstream API or SQL feature availability is inferred from the unpinned snippets. Version review is part of implementation acceptance.

## Hot/cold consistency

The source one-hour example uses 55 minutes of Parquet plus five minutes of Arrow (p. 27); this is illustrative retention. A plain concatenation of memory and file scans may double-count batches during flush. Choose a consistent catalog generation, coverage ownership or another reviewed selection mechanism so eligible events occur once. Existing snapshots must remain valid when later snapshots select cold representations. See [persistence](persistence-engine.md) for commit and compaction boundaries.

## Financial execution and results

Publish the supported SQL subset/type/function matrix rather than asserting universal ANSI compliance (VAL-026). ASOF, bucketing/OHLCV and current/as-known views require [financial semantics](financial-query-semantics.md); L1/L2/L3 remains an optional scoped extension. Flight uses a separately reviewed [client contract](flight-and-python.md); result construction and client conversion costs belong in measurements.

## Verification

Compare repeated and concurrent scans with independent expected datasets; test reordered/sparse projections, zero-column behavior if supported, predicates, limits, empty partitions, cancellation and schema mismatches. Race publication, persistence, eviction and compaction against hot-only/cold-only/hybrid queries. Include corrections and equal timestamps. Performance evidence must retain correctness checks under load (VAL-015–018/023/026).

Product trace: F03–F06/F08–F10, N02/N08; CAP-02/03/05/06. Source: pp. 6–7, 16–17, 26–28, 33–36.
