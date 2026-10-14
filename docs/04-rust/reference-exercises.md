# Reference exercises and implementation readiness

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## How to use these exercises

Read [fundamentals](fundamentals.md) first. Copy each complete Rust block in this tab into a separate learning binary and run it; blocks labeled text or interface sketches are intentionally conceptual. Standard-library examples teach ownership and invariants without choosing FDAP dependency versions. Assertions describe expected behavior, not throughput evidence.

The [ownership](ownership-and-lifetimes.md), [admission](idioms-and-errors.md), [typestate](engine-design-patterns.md), [channel](concurrency-patterns.md) and [atomic](atomics-and-memory-ordering.md) examples build from simple values to publication. Production acceptance still follows the [Engineering validation plan](../03-engineering/validation-and-traceability.md).

## Practice sequence

| Exercise | Build / change | Expected evidence | Contract |
| --- | --- | --- | --- |
| 1. Validated tick | Add checked instrument, timestamp and quantity parsing | Valid fixture preserved; malformed/range failure explicit; no silent cast | ENG-002/007 |
| 2. Bounded writer | Extend admission example to two columns | Full rejects before mutation; lengths remain equal; invalid input leaves both unchanged | ENG-003/004 |
| 3. Partial publication | Add row threshold and writer-owned timer command | Empty timer publishes nothing; partial publishes once; sustained input still services deadline | ENG-004; DEC-004 |
| 4. Retained registry | Use immutable handles and synchronized membership | Two snapshots see declared membership; old snapshot survives eviction | ENG-005 |
| 5. Bounded pipeline | Preserve unsent message on Full/disconnect | No accepted event disappears; queue and pending bytes accounted | ENG-006; DEC-006 |
| 6. Query adapter | After pinning FDAP, expose fixed fixtures | Reordered projection, residual filter and empty scans agree with oracle | ENG-001/008 |
| 7. Hot/cold transition | Represent a batch before/after catalog commit | Query counts eligible logical versions once during overlap | ENG-010/011 |
| 8. Financial versions | Add same-time trades and late amendment/cancel | ASOF/OHLCV and knowledge cutoffs match independent expected results | ENG-007/009/012 |
| 9. Client lifecycle | Add cancellable stream with owned snapshot | Disconnect/mid-stream failure releases references and reports incomplete result | ENG-013 |
| 10. Optimized coordination | Evaluate a reviewed ring against safe baseline | Wrap/full/stall invariants hold; performance run also checks correctness | ENG-003–006/014 |

Implement one step at a time. Explain its invariant in plain language before introducing an unsafe block. For a multi-column writer, validate the whole row first and design allocation/construction failure handling before publication.

## A minimal correctness harness

```rust
use std::sync::Arc;

fn snapshot(registry: &[Arc<Vec<u64>>]) -> Vec<Arc<Vec<u64>>> {
    registry.to_vec()
}

#[test]
fn readers_retain_data_after_eviction() {
    let batch = Arc::new(vec![10, 20]);
    let mut registry = vec![batch];
    let first = snapshot(&registry);
    let second = snapshot(&registry);
    registry.clear();
    assert!(snapshot(&registry).is_empty());
    assert_eq!(first[0].as_slice(), &[10, 20]);
    assert_eq!(second[0].as_slice(), &[10, 20]);
    assert!(Arc::ptr_eq(&first[0], &second[0]));
}
```

Put this in `src/lib.rs` and run `cargo test`. It verifies retained storage and repeatable acquisition over fixed membership. It does not test concurrent registry changes or a hot/cold catalog; those need synchronization and independent fixtures. Extend it with a persistence owner and show that dropping one owner does not invalidate the others.

## Verification tools and their limits

Use `cargo check`, formatting and Clippy for build/style issues and `cargo test` for observable invariants. Add explicit failure schedules: full queue, delayed reader, wrapped slot, cancellation, disk failure and partial stream. Exercise semantic overflow near limits in debug and release; checked arithmetic must not depend on build-mode panic behavior.

Miri is appropriate for supported unsafe-memory examples; compatible model-checking tools can explore synchronization schedules. Install/pin these tools only when the chosen implementation needs them. They do not prove filesystem crash consistency, real NIC integration or target-hardware latency. Fault injection/restart and workload measurements remain separate evidence.

## Performance experiment after correctness

Record toolchain/dependencies, hardware/affinity/NUMA, release profile, input schema and workload, batch/queue sizes, query concurrency, durability mode and endpoints. Separate receive-to-visible time from SQL execution and durable completion. Measure full turnover, not only append: allocation/deallocation, memory, CPU, P50/P99, loss/rejection and recovery outcomes.

Research examples of 1M ticks/s, hot P99 < 1 ms, 500 µs publication, five-minute history, 5/10 s flush and busy spinning belong to different scenarios (pp. 16–17, 21–28, 36–42). They are unverified targets/settings, not results of these exercises. Consult the [performance profiles](../00-governance/performance-profiles.md).

## Source-to-learning traceability

| Source / extracted intent | Learning pages | Safety or decision boundary |
| --- | --- | --- |
| pp. 15–17, 26–28; RES-024/025/043 | Foundations, idioms, FDAP | PRD-F01–F12; scope DEC-001/002; API DEC-010/013 |
| pp. 23–26; RES-040–042 | Foundations, schema integration | ENG-007; DEC-007/014; VAL-010/011/018 |
| pp. 29–32; RES-051–053 | Ownership, memory, atomics, lock-free | ENG-003–006; DEC-003/006; VAL-012–014 |
| pp. 33–36; RES-054/055 | Snapshots, FDAP | ENG-005/008; VAL-015–017 |
| pp. 36–39; RES-056–059 | Writer pattern, concurrency, reuse | ENG-003–006; DEC-003/004; VAL-019/020 |
| pp. 39–42; RES-060–062 | Batch coordination, leases, scheduling | ENG-005/006/014; DEC-003/011; VAL-021/022 |
| pp. 6–10, 27–28; RES-010–013/046–049 | Domain policies, persistence, clients | ENG-009–013; DEC-008/009/013/014; VAL-004/023/024/026 |

## Handoff checklist

Before integrating a lesson into the engine, name the sole mutation owner, capacity policy, publication relation, reader lifetime and reclamation boundary. Pin dependency APIs, supply fixture/oracle results and preserve accepted/visible/durable distinctions. Unsafe changes require local proof obligations and adversarial schedules; performance claims require a recorded workload.

No open decision or validation item is closed by this reference. Review the [decision register](../00-governance/decision-register.md), [technical validation](../00-governance/technical-validation.md) and [Product scope](../01-product/scope-and-mvp.md) with the relevant owners.
