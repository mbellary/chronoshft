# EXP-001: bounded SPSC queue

This isolated prototype investigates sequential queue invariants for VAL-012.
It is not an approved production queue or a resolution of DEC-003/DEC-006.

## Prototype contract

- Capacity counts individual events. Every integer N from 1 through 4096 is
  supported, with exactly N usable slots; no reserved slot or power-of-two rule.
- Zero returns `CapacityError::Zero`; larger sizes, including `usize::MAX`,
  return `TooLarge` before allocation. A failed slot reservation returns
  `AllocationFailed`. Allocation exhaustion and process-level allocator failure
  are not injected or certified by this experiment.
- Construction creates one non-cloneable producer and one non-cloneable consumer.
  Operations need mutable endpoint access. A mutex protects a preallocated ring;
  calls can wait for the mutex. This is a correctness prototype, not a lock-free
  algorithm. Poisoned mutexes panic; disconnect signaling is not implemented.
- A full push returns the original event before touching a slot. Empty pops return
  `None`. Slot positions wrap within the capacity; there are no monotonically
  increasing production counters to roll over.
- Rejecting a single full push is a provisional experiment boundary. Sustained
  overload behavior and production adoption remain assigned to EXP-002 and
  DEC-006. Concurrent publication and ownership investigation remain EXP-003.

## Reproduce

From the repository root:

```sh
cargo test --offline --locked -p exp-001-spsc-queue
```

The four integration tests correspond to TST-001 through TST-004. They compare
observations with independent identity ledgers or `VecDeque` models, and write
per-operation text reports to `CHRONOSHFT_REPORT_DIR` (default:
`target/validation-reports` relative to the repository root). The capture
plan sets this path explicitly. Failed assertions retain the observed prefix.
The wraparound schedule records seed 24301 and native index width, with a
minimum four complete write traversals for each tested capacity.

No concurrency proof, throughput, latency, allocator-failure tolerance or
architecture acceptance follows from these sequential tests. Capture results
are observations awaiting separate review.
