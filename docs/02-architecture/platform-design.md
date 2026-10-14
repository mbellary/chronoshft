# Platform design

!!! info "Research-derived design draft · 2026-10-14"
    Source: [unified HFT research notes](../assets/research/HFT-Research-Notes.pdf), pp. 8–17, 24–28 and 29–42. Ownership, catalog and resource controls below are proposed refinements where the source does not define a complete contract.

## Subsystem boundaries

| Subsystem | Owns | Boundary |
| --- | --- | --- |
| Feed adapter | Connection/framing, decoder and venue sequence state | Emits normalized events; FIX/ITCH/WebSockets require separate adapter designs |
| Ingestion writer | Current mutable Arrow builders and append cursor | Exclusive append/rotation owner; no query receives mutable builder access |
| Batch coordinator | Ready notifications and bounded admission | Publishes completed batches; notification completion is distinct from query completion |
| Active-memory registry | Retained immutable batch identities and versions | Acquires non-destructive query snapshots; tracks retirement eligibility |
| Query service | DataFusion context, providers and per-query plans | Reads eligible memory/files, applies financial semantics and returns typed results |
| Persistence worker/catalog | Parquet output and committed file/version metadata | Publishes historical visibility only under a defined commit protocol |
| Flight service | Client sessions and result streams | Handles transport, cancellation and client lifetime without holding the writer |

The source names `HftRingBufferRotator`, `HftActiveTableProvider`, `ConcurrentSplitSlot` and `ArrowBatchReadyEvent`. These are illustrative prototypes, not complete component APIs (pp. 29–42; VAL-017).

## Ownership and resource domains

**Proposed design:** each ingestion partition has one writer handle. Query workers acquire references to frozen batches from a retained registry. Persistence also holds a reference while writing. A buffer becomes reusable only when the selected retention policy and every reader/storage lease permit it. Queue slots, Arrow allocations and historical catalog entries have distinct lifetimes.

Preallocation should cover declared ring size, batch capacity and pool limits. Query concurrency, outstanding snapshots, result buffers, sorting, disk backlog and retained history need their own budgets. On exhaustion, apply the selected overload policy before writing beyond capacity; lock-free coordination does not remove backpressure (DEC-003/006).

## Financial data model

The source proposes signed 64-bit nanosecond epoch values, Arrow `Timestamp(Nanosecond, ...)` and Parquet `TIMESTAMP` with `NANOS` and UTC-adjusted semantics (pp. 24–25). Its code instead declares a non-null `Int64` timestamp, `Float64` price and `Int32` size, so that demonstration must not become the production schema silently (pp. 29–30; VAL-018).

| Field concern | Required design decision |
| --- | --- |
| Time | Epoch, timezone convention, source resolution, receive/knowledge time, overflow and invalid-value handling |
| Identity/order | Venue, instrument, partition/session, sequence and equal-timestamp tie-break rules |
| Price/quantity | Fixed-point/decimal versus floating-point, scales, units and nullability |
| Corrections | Event identity, amendment/cancel relation and system-knowledge version |
| Schema evolution | Version compatibility across Arrow, Parquet, Flight and clients |

These are proposed schema concerns, not fields approved by the PDF. Nanosecond resolution preserves precision but cannot establish event order or clock accuracy (DEC-007/014; VAL-010/011).

## Storage and lifecycle controls

The source describes micro-Parquet files on local disk, with object storage mentioned as an alternative on p. 16. Local NVMe is the reference-node baseline; object-store tiering requires its own consistency and recovery design. Partition keys, filenames, row-group sizing, compression, statistics, indexing, compaction and retention are not specified by the source.

Separate **visible-at**, **persisted-at** and **recyclable-at** states. A volatile-only mode, WAL-backed mode or source-replay recovery are alternatives awaiting DEC-008, not existing features. Private/VPC operation without runtime Internet dependencies is source intent; authentication, authorization and secret management still need concrete contracts.

## Dependencies and closure

Pin a compatible Rust/Arrow/DataFusion/Parquet/Flight set and the selected ring library before translating prototypes into APIs. Review ownership and capacity proofs, repeated snapshots, schema round trips, tier handoff and restart behavior. Related: [component contracts](component-contracts.md), [active memory](active-memory.md), [query/storage](query-and-storage.md), [technical validation](../00-governance/technical-validation.md), [decision register](../00-governance/decision-register.md).
