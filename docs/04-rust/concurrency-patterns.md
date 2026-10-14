# Concurrency patterns for ingestion and analytics

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## Concurrency vocabulary

**Concurrency** lets tasks make progress over overlapping periods; **parallelism** executes work at the same time. A thread is scheduled by the OS. An async task yields at suspension points managed by a runtime. Async does not turn CPU-heavy decoding, SQL or compression into free background work.

`Send` permits ownership transfer across threads; `Sync` permits shared references across threads. The compiler derives these properties when the constituent types permit them. Manual unsafe implementations require proof for every supported caller. An `Arc` around an unsuitable type does not fix it ([Rust Send/Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)).

## Proposed workload mapping

| Subsystem | Starting model to evaluate | Responsibility |
| --- | --- | --- |
| Network sessions | Async I/O or dedicated adapter threads | Frame/decode with bounded buffers and reconnect policy |
| Append and freeze | One dedicated owner per defined partition | Exclusive mutation and complete publication |
| Registry coordinator | Worker consuming bounded ready messages | Retained membership and snapshot synchronization |
| Query execution | Bounded query workers/runtime | Independent snapshots, scratch budgets and cancellation |
| Parquet persistence | Background bounded work | Commit, retry and durability accounting |
| Flight results | Async streaming with resource limits | Retain batches through completion/disconnect |

This is a teaching decomposition, not an approved scheduler. Partitioning must preserve venue ordering; spinning/core placement are measured decisions (research pp. 21–23, 39–42; ENG-003–006/010/013/014; DEC-003/006/011).

## Runnable ownership-transfer example

```rust
use std::{sync::mpsc::sync_channel, thread};

fn main() {
    let (tx, rx) = sync_channel::<Vec<i64>>(1);
    let writer = thread::spawn(move || {
        let frozen = vec![101, 102];
        tx.send(frozen).expect("learning receiver remains connected");
        // Dropping the last sender closes the stream after queued work.
    });
    let received = rx.recv().expect("one batch");
    writer.join().expect("learning writer completed");
    assert_eq!(received, vec![101, 102]);
    assert!(rx.recv().is_err());
}
```

The channel moves the vector to its receiver. A queue bound of one limits queued messages, not the total bytes in the engine. `send` may block when full; production must handle disconnect and overload. This standard-library channel is a correctness exercise, not a lock-free or HFT performance recommendation. Its receiver consumes messages; populate a registry before queries acquire snapshots ([sync_channel](https://doc.rust-lang.org/std/sync/mpsc/fn.sync_channel.html)).

## Backpressure and fairness

**Backpressure** means downstream capacity affects upstream admission. A nonblocking send can return Full with the unsent value; retain that ownership and apply the selected bounded retry/reject/disconnect policy. Never report acceptance after losing the event. Waiting forever may be incompatible with a live feed; one `spin_loop()` call does not resolve saturation.

Use row, byte and age limits. Bound simultaneous queries, snapshot retention, result backlog and persistence work as well as input. A timer request must be handled by the writer even during sustained input. Schedule publication from elapsed monotonic time or a reviewed control-message policy; market timestamps are not elapsed-time clocks.

## Locks and async suspension

A `Mutex` protects exclusive access; a `RwLock` distinguishes readers and writers. Long queries should operate on cloned immutable handles outside the membership lock. Guard fairness and contention are implementation choices to measure. Handle poisoning or fail under an explicit recovery policy.

Avoid carrying a synchronous mutex guard across `.await`; capture the needed owned data, release the guard, then await. Blocking I/O or compression on async worker threads can stall unrelated work; use a bounded blocking/dedicated worker strategy. Async cancellation may drop a future at a suspension point, so partial commits and resource ownership need explicit state handling.

## Shutdown sequence to define

1. Stop or reject new admission and cancel feed/timer producers.
2. Writer resolves its partial batch under the selected policy.
3. Close ready senders and drain or explicitly abandon remaining work.
4. Finish or cancel queries and client streams, releasing retained references.
5. Complete the selected persistence boundary or report accepted-but-uncommitted events.
6. Join workers and report failures; a detached thread is not evidence of clean shutdown.

Order may change with the approved durability mode. Never await persistence while holding a lock that its worker needs. Test shutdown with a full queue, a stalled reader, a mid-stream disconnect and a failing disk.

Related: [ingestion](../03-engineering/ingestion-engine.md), [Flight](../03-engineering/flight-and-python.md), [memory](memory-management.md), ENG-002–006/010/013. Source publication and wait settings remain examples; no scheduler/freshness SLA is established here.
