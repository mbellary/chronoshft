# Ownership and lifetimes for market-data buffers

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## The question ownership answers

For every buffer ask: who may mutate it, who keeps it alive, and when may its allocation be reused? In ChronosHFT, the proposed answer is one mutable writer followed by immutable references retained by queries and persistence (research pp. 29–42; ENG-003–006; DEC-003).

A move transfers responsibility for a value. A borrow gives temporary access: `&T` for shared inspection and `&mut T` for exclusive mutation. Do not work around a borrowing error with raw pointers; first examine whether the API gives two actors responsibility for the same buffer.

## Borrow a packet while decoding, own the event afterward

```rust
fn payload<'a>(packet: &'a [u8]) -> Option<&'a [u8]> {
    packet.get(2..)
}

fn main() {
    let packet = vec![0, 2, 10, 20];
    let view = payload(&packet).expect("fixture has a header");
    assert_eq!(view, &[10, 20]);
}
```

`'a` links the returned view to the input packet. It does not extend storage lifetime. A decoder may inspect this borrowed slice synchronously; it must create owned event fields or transfer an owned buffer before another task reuses the network memory. Returning a slice of a local `Vec` is rejected because the allocation would disappear when the function returns.

A `Vec` move transfers its ownership without cloning its elements. `Clone` explicitly asks for a copy according to the type's implementation; cloning a `Vec<T>` copies elements. `Copy` is for simple values that may be duplicated implicitly. Do not derive it for an owning batch or resource lease.

## Freeze, share, retire

```rust
use std::sync::Arc;

fn main() {
    let writing = vec![101_i64, 102];
    let frozen = Arc::new(writing); // Move storage into shared ownership.
    let mut registry = vec![Arc::clone(&frozen)];
    let query_snapshot = registry.clone(); // Clone handles, not row vectors.
    let persistence = Arc::clone(&frozen);
    registry.clear(); // Remove eligibility for new snapshots.
    drop(frozen);
    drop(persistence);
    assert_eq!(query_snapshot[0].as_slice(), &[101, 102]);
}
```

The query still owns valid data after registry removal. This teaching example uses `Vec`; production uses immutable Arrow batches and associated metadata. Shared ownership alone does not establish which batches belong in a consistent snapshot. Synchronize snapshot acquisition and membership changes under a reviewed registry/catalog protocol.

`Arc<T>` keeps a shared allocation alive. Its thread safety depends on the contained type, and it does not authorize concurrent mutation. Its final release may destroy a large allocation on the releasing thread. Budget reader lifetimes and destruction costs; immutable sharing is not automatically bounded memory.

## Lifetime traps in the engine

| Trap | Better ownership boundary |
| --- | --- |
| Timer and append both borrow builders mutably | Timer sends a request; writer alone rotates |
| Query returns a borrowed batch from a temporary lock guard | Acquire owned immutable references while guarded |
| Callback hands data to query, then marks slot reusable | Keep allocation ownership until every downstream reader releases it |
| Feed task borrows a receive buffer across reuse | Decode into owned event fields or transfer an owned packet |
| Worker requires `'static`, so memory is leaked | Transfer owned captures; `'static` bound prohibits short-lived borrows and does not mean the value must live forever |

`Rc` shares ownership within one thread; use `Arc` for eligible data shared across threads. `RefCell` checks borrowing at runtime within a suitable single-thread context; it is not a substitute for inter-thread synchronization.

## Reuse is a separate protocol

Returning a queue slot does not return every Arrow allocation it once referenced. Builder finishing can transfer values into arrays; `reserve()` can allocate replacement memory (VAL-014/020). A reusable pool needs explicit leases and generation rules. Do not use a sampled reference count as an independent permission to write raw storage while other actors may acquire references.

Trace: [active-memory lifecycle](../03-engineering/active-memory-lifecycle.md), [component contracts](../02-architecture/component-contracts.md), ENG-003–006, VAL-013–015/019–021. Practice: retained snapshot in [exercises](reference-exercises.md). Language sources: [Ownership](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html), [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html).
