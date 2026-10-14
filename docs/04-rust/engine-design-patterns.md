# Rust design patterns for the engine

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## Single owner and message passing

Give each mutable builder one writer. Adapters supply normalized events, timer tasks request publication and the writer makes all append/rotation decisions. This actor-like pattern means messages cross ownership boundaries while mutation stays local. It addresses the source public `&self` methods that do not enforce single-caller discipline (pp. 29–39; VAL-013/019).

Do not give a timer a shared mutable builder merely to flush it sooner. Define bounded control/event queues and scheduling precedence so a busy feed cannot starve publication or shutdown (ENG-003/004/006; DEC-004/006).

## Strategy traits for adapters

A trait isolates venue-specific decoding from admission and storage. Example interface sketch, not a complete adapter:

```text
trait Decoder {
    decode(&mut self, packet: &[u8]) -> Result<NormalizedEvent, DecodeError>
}
```

The decoder owns framing/session state; the resulting event owns the fields required after packet reuse. A generic `Ingest<D: Decoder>` can specialize a selected adapter. A `Box<dyn Decoder + Send>` can support runtime selection when object-safety and thread-transfer rules permit it. Preserve sequence scope and provenance regardless of dispatch choice (ENG-002/007; DEC-002/014).

## Typestate for freeze boundaries

**Typestate** represents different valid stages with different Rust types. A consuming `freeze(self)` prevents continued append through that old writer value:

```rust
struct Writing { rows: Vec<i64> }
struct Frozen { rows: Vec<i64> }

impl Writing {
    fn append(&mut self, value: i64) { self.rows.push(value); }
    fn freeze(self) -> Frozen { Frozen { rows: self.rows } }
}

fn main() {
    let mut writer = Writing { rows: Vec::new() };
    writer.append(42);
    let batch = writer.freeze();
    assert_eq!(batch.rows, vec![42]);
    // writer.append(43); // Would fail: writer was moved into freeze.
}
```

Keep frozen storage private and expose read-only APIs. The example illustrates the transition; it omits production bounds and schema checks. Typestate can prevent misuse of a handle but cannot prove that a file survives power loss. A `Committed` type needs a commit protocol with evidence, not a rename of `Written` (ENG-010; DEC-008).

## Immutable snapshot registry

Separate a consuming notification queue from the retained query registry. A coordinator consumes ready messages once and inserts immutable batch references. Each query acquires fixed membership and references without draining the queue. The registry/catalog handoff must select logical event versions exactly once when memory and files overlap (ENG-005/011/012).

A safe initial teaching design uses a short mutex around membership changes and snapshot capture. Clone references while guarded and scan after releasing the lock. This adds synchronization at a control boundary and requires measurement; it does not establish a production performance choice. Never hold the guard for SQL execution or disk I/O.

## Leases, generations and RAII

A **lease** keeps storage in use while a reader needs it. RAII can release the lease on normal return, error or unwinding. A **generation** distinguishes successive occupants of a reused slot: `(slot 3, generation 8)` differs from `(slot 3, generation 9)`. The acquisition protocol must verify and retain the generation atomically with ownership; comparing stale numbers alone cannot keep memory alive.

This matters for delayed queries, persistence retries and slow Flight clients. Registry eviction changes future membership but existing leases remain valid. A pool must apply pressure policy when no allocation is free (ENG-005/006; DEC-003/006).

## Persistence and domain strategies

Hide storage operations behind a boundary that returns explicit write/commit errors and retains retry identity. Queries select a catalog snapshot; they do not infer durability from channel acknowledgment. Apply the selected recovery and correction model consistently across hot/cold paths.

For ASOF and OHLCV, start from an independent semantic oracle. Match keys, tie-breaks, bucket boundaries and knowledge cutoffs belong in typed domain policy, not hidden assumptions inside iterator order. No Rust pattern chooses these financial rules for the team (ENG-009/012; DEC-009/014).

## Pattern selection checklist

| Problem | Start with | Evidence before optimizing |
| --- | --- | --- |
| Builder aliases | Exclusive writer handle | Append/timer concurrency and complete-row tests |
| Multiple query readers | Immutable retained snapshots | Repeated scans and delayed-reader survival |
| Overlapping memory/files | Consistent coverage catalog | Handoff fixtures with no gaps/double counts |
| Adapter variability | Trait plus owned normalized event | Replay, gaps and malformed fixtures |
| Pool pressure | Leases plus bounded accounting | Cancellation, slow readers and generation reuse |

Source: pp. 29–42; proposed refinement from [engineering design](../03-engineering/engineering-design.md), [active memory](../02-architecture/active-memory.md) and [query engine](../03-engineering/query-engine.md). DEC-003 remains unresolved; these patterns are teaching choices, not approved ADRs.
