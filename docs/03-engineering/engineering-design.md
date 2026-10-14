# Engineering design

!!! info "Research-derived engineering draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages), the [Architecture tab](../02-architecture/system-architecture.md) and [Product requirements](../01-product/product-requirements.md). Source intent, existing validation findings and proposed engineering refinements are distinguished below. No implementation, benchmark result or decision approval is asserted.


| Control | Value |
| --- | --- |
| Document | DOC-07 |
| Status | Detailed design proposal; implementation and review pending |
| Requirement baseline | [DOC-06 / ENG-001–015](engineering-requirements.md) |

## Design basis and responsibility

The PDF describes `HftRingBufferRotator` (pp. 29–32), `HftActiveTableProvider` (pp. 33–36), `ConcurrentSplitSlot` (pp. 36–39) and `ArrowBatchReadyEvent` (pp. 39–42). These illustrate the intended Arrow/Disruptor/DataFusion path. The [platform design](../02-architecture/platform-design.md) assigns writer, registry, query, persistence and client ownership. This design refines the prototypes using the existing VAL register; it does not reproduce unsafe listings as runnable code.

## Proposed end-to-end algorithm

1. Decode and normalize an event with declared venue/schema/sequence provenance.
2. Apply gap/duplicate/lateness policy and reserve bounded writer capacity before mutation.
3. Append every column under exclusive writer ownership; on row threshold or writer-handled timer, freeze initialized rows into an immutable batch.
4. Publish its stable identity, schema/version and ordering metadata. A coordinator inserts it into a retained registry; the notification queue is not the queryable table.
5. A query captures a consistent hot/catalog view, acquires immutable references, selects eligible logical versions once, then plans projection/filter/financial operations.
6. A background worker holds the batch through Parquet write and the selected commit protocol. Publish committed metadata at the defined visibility boundary.
7. Retire hot entries according to retention. Reclaim allocations only after every query, result and persistence owner has released them and the selected reuse protocol allows it.

Steps 2–7 contain proposed safety/consistency refinements (ENG-003–006/010–012), particularly where the source polls a consuming queue or treats Disruptor completion as query completion.

## Data structures and interface intent

| Proposed structure | Contents / owner | Invariant |
| --- | --- | --- |
| Adapter state | Framing, session/partition sequence and decoder buffers | Venue-specific ordering and error state stay explicit |
| Writer handle | Mutable columns, append cursor, active generation | One append/rotation owner; no exposed shared mutable builder |
| Ready event | Batch identity and immutable reference | Notification slot lifetime does not authorize Arrow allocation reuse |
| Registry entry | Batch reference, schema and coverage/version metadata | Acquisition is non-destructive; eviction does not invalidate existing readers |
| Query snapshot | Hot references and eligible committed file/catalog view | Fixed membership/cutoff for the declared consistency model |
| Persistence task | Batch lease, output state and commit outcome | Failures cannot advertise partial history as committed |

These are conceptual structures, not a pinned Rust API. Select schema and version metadata in [schemas and sequencing](event-schema-and-sequencing.md) and [persistence](persistence-engine.md).

## Ownership alternatives to evaluate

| Alternative | Benefit | Design cost / evidence |
| --- | --- | --- |
| Transfer builders into immutable batches | Clear Arrow ownership; readers retain immutable arrays | Turnover may allocate; bound and audit it rather than claiming zero allocation |
| Bounded reusable allocation pool | Candidate for controlled turnover | Prove layout, generation, lease and reclamation behavior; delayed readers create pool pressure |
| Double buffering inside a slot | Candidate for partial-batch freshness | Two buffers alone cannot bound long-reader retention or enforce caller exclusion |
| Disruptor batch coordination | Separates batch publication from tick append | Pin crate/API/wait strategy; retain Arrow lifetime accounting outside event acknowledgment |

DEC-003 selects ownership. No alternative is approved here. Start with correctness evidence before raw-pointer optimizations; any unsafe block needs local invariants and an applicable verification plan.

## Resource and scheduling design

Budget queue storage, retained history, outstanding snapshots, query scratch/sort memory, results, persistence backlog and compaction separately. Total memory includes detached Arrow arrays, not just ring builder capacity. Choose overload and cancellation behavior before allowing unbounded reader/persistence retention.

Source core pinning and BusySpin are reference proposals (pp. 21–23, 42). Record NUMA, SMT, IRQ placement and contention; keep logging/I/O outside the audited append path where feasible. DPDK is adapter-specific and is not a replacement for WebSocket framing/TLS/JSON decoding (VAL-009/025).

## Engineering guide

- [Ingestion engine](ingestion-engine.md): decoding, admission and publication.
- [Active memory lifecycle](active-memory-lifecycle.md): ownership, ordering and reclamation.
- [Query engine](query-engine.md): stable scans and hybrid selection.
- [Persistence engine](persistence-engine.md): commit, corrections and restart.
- [Event schema and sequencing](event-schema-and-sequencing.md): cross-layer type/order contracts.
- [Financial query semantics](financial-query-semantics.md): oracle-ready operator definitions.
- [Flight and Python integration](flight-and-python.md): typed transport and client resources.
- [Validation and traceability](validation-and-traceability.md): evidence, source map and decision handoff.
