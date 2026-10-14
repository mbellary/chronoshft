# Rust at the Arrow, DataFusion, Parquet and Flight boundaries

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## Learn each library's role

| Foundation | Engine role | Rust ownership concern |
| --- | --- | --- |
| Arrow | Typed columnar active/result batches | Freeze builders; retain immutable arrays |
| DataFusion | SQL planning/execution over providers | Acquire snapshot before plan/stream outlives scan call |
| Parquet | Historical columnar files | Hold input through write and commit |
| Flight | Typed result transport | Hold result/snapshot through stream or cancellation |

Research pp. 8–13, 15–17, 26–28 establishes this direction; it does not provide a pinned compatible manifest. Integration interfaces below are conceptual. Choose compatible Arrow/DataFusion/Flight/Parquet versions, keep Cargo.lock and compile against their exact official documentation before supplying production code (ENG-001; DEC-010/013; VAL-017).

## Arrow schema and construction

Normalize and validate events first, append all columns under one mutable writer, then construct an immutable batch with consistent lengths and schema. A batch has no knowledge of venue gaps or financial correction rules unless the engine preserves the required fields and metadata.

The source uses `Int64` timestamp in its sample despite proposing `Timestamp(Nanosecond, ...)` elsewhere (pp. 23–26, 29–30). Select explicit epoch/unit/timezone semantics and checked conversions. Define fixed-point/decimal scale, quantity unit, nullability, identity, source sequence and schema version in [Engineering schema](../03-engineering/event-schema-and-sequencing.md) before committing an Arrow layout (ENG-007; DEC-007/014).

Treat in-process sharing as a boundary-specific property. Row normalization, batch construction, sorting, new result columns and wrappers may allocate or copy. No end-to-end zero-copy or no-allocation guarantee follows from immutable Arrow references.

## DataFusion provider adapter

A `TableProvider` connects query planning to eligible data; an execution plan describes how to obtain result batches. A scan must acquire retained snapshot membership rather than polling the ingest queue. The snapshot owns its batches and committed-file selection through execution.

```text
scan(request):
    acquire consistent hot/catalog snapshot
    select eligible event/version coverage once
    validate schema and supported pushdowns
    construct plan with owned references
    execute and stream; release on completion/cancellation/error
```

Projection selects/reorders columns. Apply it at one defined layer and keep plan indices, batch columns and output schema consistent; `[2, 0]` is a better fixture than selecting every column. Unsupported predicates must remain for residual evaluation; do not claim pushdown without matching semantics. Empty scans retain their typed schema. Limits need care with partitions, ordering and financial operators (ENG-008; VAL-015–017/026).

Rust traits keep this adapter boundary explicit, but trait signatures and plan constructors are version-dependent. The source `MemoryExec` listing is not a current API recipe. See [query engine](../03-engineering/query-engine.md).

## Parquet commit and hybrid handoff

A persistence task owns immutable input until its outcome is resolved. File completion, durability and catalog visibility are distinct states. A plain channel acknowledgment or file writer return does not approve durable admission. Select the failure model and recovery mechanism under DEC-008.

When both a hot batch and its file exist, select eligible logical versions exactly once using the reviewed consistent catalog/coverage protocol. A snapshot acquired before hot retirement remains valid. Compaction must preserve current and as-known semantics and defer deletion needed by active snapshots (ENG-010–012; VAL-023/024).

## Flight and Python resource lifetime

A result stream may outlive the query request handler. Transfer owned references into the stream; do not borrow temporary local batches. Propagate cancellation to execution and release retained memory on disconnect, timeout and error. A slow client requires byte/concurrency limits (ENG-013).

Flight crosses processes using transport and Arrow IPC; dataframe conversion is another boundary. Verify ns timestamps, nulls and chosen numeric precision independently for each supported client. The PDF does not establish Flight SQL or an existing `chronoshft` Python package. State request, result and partial-stream contracts before choosing APIs (DEC-013; VAL-004).

## Financial correctness stays explicit

Rust types and library integration do not resolve ASOF match keys/direction/tolerance, OHLCV equal-time order or correction knowledge cutoffs. Preserve enough identity/ordering/version data to implement the ratified contract. Validate against independent oracles across hot-only, cold-only and hybrid snapshots (ENG-009/012; DEC-009/014).

Source map: pp. 6–10, 15–17, 23–28, 33–36; PRD-F03–F12; ENG-001/004/005/007–013. Companions: [query](../03-engineering/query-engine.md), [persistence](../03-engineering/persistence-engine.md), [financial semantics](../03-engineering/financial-query-semantics.md), [Flight/Python](../03-engineering/flight-and-python.md).
