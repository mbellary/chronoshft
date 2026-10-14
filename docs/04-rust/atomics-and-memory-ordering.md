# Atomics and memory ordering

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## What atomics solve

An atomic operation accesses one atomic value without a torn concurrent read/write. It does not make an entire Arrow batch atomic, enforce one producer, validate capacity or retain allocation lifetime. Those are protocol obligations (research pp. 30–32, 37–39; ENG-003–005; VAL-013/019).

**Happens-before** describes an ordering relationship by which synchronization makes earlier work visible to another thread. Before choosing an ordering, write the data being protected, its exclusive owner and the transition that makes it readable.

## Ordering vocabulary

| Ordering | Meaning to reason about | Engine use to review |
| --- | --- | --- |
| Relaxed | Atomic value access without cross-thread publication ordering | Independent telemetry counters |
| Release | Publishes preceding work through an appropriate synchronization relation | Announce initialized slot state |
| Acquire | Observes preceding published work when paired through the observed atomic value | Consume a published slot |
| AcqRel | Acquire and release effects for read-modify-write operations | Reviewed state transitions requiring both |
| SeqCst | Stronger ordering with a total order of sequentially consistent atomic operations | Simpler starting reasoning where suitable; still needs ownership |

Loads cannot use Release and stores cannot use Acquire. Read-modify-write and compare-exchange have distinct success/failure rules; do not select them by trial and error. Consult [Rust Ordering](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html) for operation-specific semantics.

## Publication handshake as a protocol sketch

```text
Producer                         Consumer
claim a free slot exclusively
initialize all row/column data
store ready sequence: Release -> load ready sequence: Acquire
                                 inspect or transfer initialized data
                                 finish access to reusable slot
load free sequence: Acquire   <- store free sequence: Release
reuse only after confirmed completion
```

The consumer must observe the relevant publication before accessing non-atomic payload. The reverse relation is needed before producer reuse. If the consumer hands immutable detached storage downstream, completion releases the notification slot only; downstream storage remains owned separately. Acquire on an unrelated flag does not establish this relation.

## Safe atomic teaching example

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

fn main() {
    let published_rows = AtomicU64::new(0);
    let ready = AtomicU64::new(0);
    thread::scope(|scope| {
        scope.spawn(|| {
            published_rows.store(128, Ordering::Relaxed);
            ready.store(1, Ordering::Release);
        });
        while ready.load(Ordering::Acquire) == 0 {
            std::hint::spin_loop();
        }
        assert_eq!(published_rows.load(Ordering::Relaxed), 128);
    });
}
```

All shared values are atomic and publication is one-shot. Scoped threads allow these borrows to finish before the scope returns. This demonstrates a relation, not an Arrow slot, timeout policy or reusable ring. Repeating it with the same ready value requires a new protocol for identity, state and reuse. Do not replace the atomic payload with unsynchronized shared mutable data.

## Counter and identity rules

Use Relaxed counters only when they are independent observations, not readiness or reclamation signals. A metric reading does not create a coherent snapshot across several counters. Never infer a complete batch from row count alone.

A ring needs full/empty discrimination and a reviewed wraparound rule. Subtraction/comparison near integer wrap can fail if ordinary arithmetic is assumed. Require a bounded outstanding distance and prove any modular arithmetic. Slot indices identify locations; generation/sequence numbers identify occupants. The ABA problem occurs when a value returns to the same representation despite intervening changes.

## Review questions before implementation

What data precedes publication? Which load observes it? Who owns append/flip? What prevents two callers from entering the same slot? What stops reuse during a delayed read? Which generation is observed? What happens on full capacity, shutdown or failure?

Use the [lock-free review](lock-free-patterns.md) and [active-memory lifecycle](../03-engineering/active-memory-lifecycle.md). Appropriate tools can explore interleavings, but neither stronger orderings nor a passing stress test proves the whole protocol. DEC-003 and VAL-012/013/019–021 require closure.

## Ring-buffer ordering contract

For the proposed single-producer/single-consumer ring, the synchronization points are mandatory:

| Operation | Required ordering | Protected boundary |
| --- | --- | --- |
| Producer checks consumer reclamation sequence before claiming/reusing a slot | `load(Ordering::Acquire)` | Prior consumer payload access has finished |
| Producer publishes the sequence after initializing the entire event payload | `store(Ordering::Release)` | All payload writes precede visibility of readiness |
| Consumer checks the producer sequence before touching the payload | `load(Ordering::Acquire)` | Observing that publication makes the initialized payload visible |
| Consumer advances reclamation sequence after moving the owned batch and finishing slot access | `store(Ordering::Release)` | Slot access completes before producer reuse |

These Acquire/Release operations are the memory-ordering barriers; no separate `atomic::fence` is required for this handshake. A `Relaxed` readiness/reclamation cursor is forbidden. A standalone fence, `SeqCst`, volatile access or cache alignment does not replace the ownership protocol. Observe the matching generation, not just a reused slot index. Multi-consumer reuse waits for every relevant gating sequence.

When using `disruptor-rs`, its pinned implementation must provide equivalent publication/consumer/gating relations; audit the actual crate source and retain the version and review before accepting the adapter. Do not claim these custom cursor calls exist inside an unaudited dependency. Registry publication subsequently uses the safe `ArcSwap` API; do not build a raw-pointer substitute. Queries acquire an owned immutable snapshot, never ring payload references after reclamation. See the [visibility boundary](../02-architecture/active-memory.md#memory-visibility-boundary).

Required verification includes modeled interleavings for publication/reuse and stress tests on weakly ordered hardware where available. Validate complete columns, generation identity and delayed readers across wraparound. An x86-only stress pass cannot establish the protocol's correctness (VAL-013/019/021).
