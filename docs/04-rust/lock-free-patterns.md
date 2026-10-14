# Lock-free patterns and unsafe review

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## Learn the progress claim before using it

**Blocking** synchronization may wait for another participant. **Lock-free** progress means some operation progresses despite stalled participants; **wait-free** progress gives each operation a bounded-step progress guarantee under the stated model. A try-operation that immediately returns Full does not promise successful admission. Absence of a mutex does not establish any complete progress or latency guarantee.

The source proposes SPSC rings, double buffering and Disruptor coordination (pp. 29–42; RES-051–062). These are candidates under DEC-003. Begin with the safe ownership-transfer exercises; custom unsafe memory is an advanced optimization requiring evidence.

## SPSC ring protocol

SPSC means **single producer, single consumer**. For a reviewed design:

1. Validate nonzero capacity, mask assumptions and full/empty convention during construction.
2. Issue distinct restricted producer/consumer handles; methods requiring `&mut self` prevent concurrent calls through the same safe handle.
3. Check capacity and slot ownership before appending any payload.
4. Initialize exactly the valid row count and publish through the reviewed Release/Acquire relation.
5. Consumer takes or inspects the item only after observation.
6. Release the notification slot only when its local access has ended; retain downstream batch ownership independently.
7. Check sequence wrap, disconnect, panic, partial initialization and destruction paths.

A bitmask `index & (N - 1)` requires power-of-two N; other designs may use modulo. One-empty-slot and per-slot-sequence designs have different capacities. Pick one and document it instead of mixing conventions. A queue of batch notifications is distinct from five minutes of retained query history.

## Why UnsafeCell is not a lock

`UnsafeCell<T>` permits interior mutation beneath a shared reference. It does not legalize data races or overlapping exclusive references. `unsafe` means the author must establish obligations the compiler cannot check; it does not disable those obligations ([UnsafeCell](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html)).

Every raw access needs valid alignment, initialized memory, bounds, lifetime, aliasing and synchronization reasoning. Manual `unsafe impl Sync` must cover every allowed caller, including concurrent append/flush/poll and error paths. A comment saying “single producer” is insufficient if a public API permits multiple safe callers.

## Research examples: retain intent, repair gaps

| PDF example / pages | Useful idea | Required refinement / validation |
| --- | --- | --- |
| `HftRingBufferRotator`, 29–32 | Batched columnar ingestion with sequence publication | Full check precedes writes; enforce caller discipline and row bounds; VAL-012/013 |
| Builder `finish`/`reserve`, 31–32 | Transfer initialized values to immutable arrays | No automatic capacity-return guarantee; measure allocation and define ownership; VAL-014 |
| `HftActiveTableProvider`, 33–36 | Bridge active data to SQL | Use retained snapshots rather than consuming queue; apply projection once; VAL-015–017 |
| `ConcurrentSplitSlot`, 36–39 | Alternate writing and published work | Writer owns append and flip; track allocation lifetimes and capacity; VAL-019/020 |
| `ArrowBatchReadyEvent`, 39–42 | Batch-level coordination | Callback acknowledgment is not query completion; keep leases; VAL-021 |
| BusySpin/alignment, 41–42 | Explore signaling/cache costs | Verify chosen crate/platform and measure CPU/latency; VAL-022 |

Do not assert that every restricted use of the ring is inherently racy. The documented issues are unenforced caller discipline and incomplete capacity/lifetime reasoning. Finishing a builder can detach storage, so a query does not necessarily read the mutable builder's current allocation. Two buffers still cannot guarantee availability for indefinitely slow readers.

## Reclamation and ABA

A stale slot pointer may refer to a newer occupant after wraparound. Generations help detect this but require an acquisition protocol that keeps the observed allocation alive. Candidate approaches include immutable shared ownership, explicit pool leases and reviewed epoch/hazard protocols. Never improvise a raw pointer pool with a reference-count sample as its safety proof.

Disruptor completion gates its own event slots according to its protocol; it cannot infer that a DataFusion plan or Flight stream has finished with handed-off Arrow data. Retained allocations need a separate ownership boundary (ENG-005/006).

## Evidence before accepting an unsafe optimization

Write invariants and a concrete adversarial schedule. Test full queues, repeated wraps, stalled consumer, delayed query, concurrent allowed callers, timer races, cancellation and destruction. Miri can help detect undefined behavior in exercised code; a compatible model-checking harness can explore synchronization interleavings. Neither covers all production/hardware behavior, and tooling must support the chosen implementation.

Compare with the safe baseline using the same correct workload. Measure publication and reclamation, allocations, tail latency, CPU use and losses; an `#[inline(always)]`, raw copy or busy spin is not performance evidence. Source pointer-copy advice on p. 32 is not permission to bypass bounds without proof.

Links: [technical validation](../00-governance/technical-validation.md), [engineering lifecycle](../03-engineering/active-memory-lifecycle.md), [performance profiles](../00-governance/performance-profiles.md), [exercises](reference-exercises.md). ENG-003–006/014; DEC-003/006/011 remain pending.
