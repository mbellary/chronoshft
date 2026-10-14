# Memory management for bounded HFT workloads

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## Stack, heap and destruction

Small local values and handles can live in stack frames; vectors and strings own heap storage. Moving a handle does not duplicate its payload. Capacity reservation prepares storage but does not bound how many items your API admits. Rust ownership provides deterministic release points, while allocator, page-fault and destructor costs still need measurement (research pp. 10–11, 29–39; VAL-005/014).

For `Vec`, length counts initialized elements and capacity counts available element storage. Access only initialized elements. `clear()` removes elements while keeping vector capacity; moving storage out leaves replacement work to the writer. Use fallible reservation where startup/capacity failure is part of the contract. No tracing GC means no tracing collector pause, not guaranteed constant append or release latency.

## Rows versus columns

A row-oriented `Vec<Tick>` is an approachable decoding/staging example. Arrow stores separate typed columns, supporting analytical scans over selected fields. A valid batch needs equal column lengths, matching schema/nullability and complete initialized rows. Validate first, then append under one owner; never publish halfway through a multi-column row (ENG-003/004/007).

The source's timestamp/price/size sample uses 8 + 8 + 4 = 20 value bytes per row. `1,024 × 4,096 = 4,194,304` rows need about 80 MiB for those values alone. At 1M rows/s this is about 4.19 s of input without drain, not five minutes of retained history. Double buffers, validity bits, schemas, references, metadata and outstanding detached arrays add memory. The sample is not the final financial schema (VAL-018).

## Budget the whole lifetime

| Allocation family | Why it remains resident | Limit to define |
| --- | --- | --- |
| Network/decoder buffers | Partial frames, reconnect and staging | Bytes/session and session count |
| Mutable builders | Active partial batches | Writers × columns × capacity |
| Ready messages | Coordinator backlog | Message count and referenced bytes |
| Retained batches | Recent history and outstanding readers | Retention bytes/age and snapshot leases |
| Query scratch/results | Sorts, joins and slow clients | Concurrent work, bytes and deadlines |
| Persistence/compaction | Write/retry and old versions | Backlog bytes and eligible file retention |

Do not add two owners' references as if they were two separate payload allocations. Conversely, a detached allocation remains resident after its builder slot is free. Account for unique live payloads plus their metadata and scratch. A queue bound alone does not constrain a large message or a retained query (ENG-006).

For capacity planning, multiply event rate by retention and measured resident bytes/event, then add headroom and subsystem budgets. Five minutes at 1M rows/s gives 300M rows, or roughly 5.59 GiB for the sample's 20-byte values alone. Actual schema, compression and client scratch determine real requirements.

## Transfer and reuse alternatives

**Immutable transfer** moves completed storage into a batch and allocates or acquires replacement capacity. It gives a clear reader lifetime and a safe starting design. Measure rotation as well as append.

**Reusable pools** hold a finite number of allocations and lend them through leases. A free builder slot and a free payload allocation are different resources. Return storage only after all relevant query/result/persistence owners have released it. Pool exhaustion requires the chosen overload policy, not silent overwrite.

Builder `finish()` does not establish that dropping an Arrow batch returns capacity to the old builder. The existing source validation rejects that unconditional claim; pin the actual Arrow version and audit allocation behavior (VAL-014/020; DEC-003).

## Cache layout and platform tuning

False sharing occurs when independent frequently written values share a coherence cache line. Padding sequence counters may reduce it, but 64-byte alignment is a source example, not universal platform evidence. Separate hot mutable metadata from cold configuration; measure before increasing every struct's size.

Preallocation does not imply prefaulted pages or NUMA locality. Record affinity, SMT, IRQ placement and memory placement when measuring. Core pinning/busy spin can consume resources and compete with query/persistence work; use workload-qualified evidence (DEC-011; VAL-009/022).

## Audit append and turnover separately

Measure decoding, append, partial/full freeze, Arc/reference creation, notification, registry eviction and final destruction. Include repeated rotations, low-volume timers, stalled readers, disk backlog and cancellation. Report allocation/deallocation counts, peak resident memory, page faults, CPU and P99 with event accounting. Never infer no allocation from a single warmed-up append loop.

Sources: [engineering lifecycle](../03-engineering/active-memory-lifecycle.md), [requirements](../03-engineering/engineering-requirements.md), [performance ledger](../00-governance/performance-profiles.md), [Vec documentation](https://doc.rust-lang.org/std/vec/struct.Vec.html). ENG-003–006/014; DEC-003/006/011; VAL-005/012/014/020.
