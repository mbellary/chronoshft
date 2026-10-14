# Component contracts

!!! info "Proposed contracts from unified research · 2026-10-14"
    The [HFT research notes](../assets/research/HFT-Research-Notes.pdf) describe component intent on pp. 8–17 and prototypes on pp. 29–42. The contracts below fill documented gaps; they remain proposals linked to the existing decision register.

## Interface responsibilities

| Boundary | Input → output | Required behavior / unresolved choice |
| --- | --- | --- |
| Feed → adapter | Venue packets/messages → normalized events or errors | Preserve source precision and sequence; declare framing, reconnect, gap, duplicate and malformed-message policies (DEC-002/014) |
| Adapter → writer | Validated event → admission outcome | Check capacity before append; return accepted/rejected/backpressured state with accounting; choose full-ring policy (DEC-006) |
| Writer → coordinator | Completed column buffers → batch-ready notification | Publish only initialized immutable data with identity, schema/version and ordering metadata; enforce one writer (DEC-003) |
| Coordinator → registry | Batch reference → retained entry | Insertion does not transfer sole ownership to the next query; define retention and memory limits (DEC-003/004) |
| Registry → query provider | Snapshot request → immutable batch/file view | Non-destructive scans; stable identities and schema; define snapshot cutoff and concurrent-query semantics (DEC-003/009) |
| Registry → persistence | Frozen batch → commit outcome | Hold ownership through write; distinguish in-progress file from query-visible committed file; report failure (DEC-008) |
| Catalog → hybrid planner | Query interval/version → eligible hot/cold sources | Cover each logical event/version once during handoff; honor corrections and knowledge time (DEC-009) |
| Query → Flight/client | Typed result → stream completion/error | Define types, authentication, cancellation, partial-stream errors and client conversion behavior (DEC-013) |
| Reader/storage → reclamation | Released lease/commit → reuse eligibility | Queue acknowledgment alone cannot authorize reuse while a query still retains buffers (DEC-003) |

## Publication and acknowledgment

**Accepted** means an event passed the declared admission boundary. **Published** means a completed batch is eligible for a new query snapshot. **Durable** means the selected failure model and commit protocol protect it. **Recyclable** means no outstanding owner can read the backing allocation and retention permits removal. The source does not define a durable ingest acknowledgment; implementations must not treat these outcomes as interchangeable (VAL-023/024).

## Backpressure and failures

A bounded ring can fill when queries or persistence lag. Candidate policies are bounded waiting/backpressure, rejection/drop with explicit counters, or disconnect and replay where the feed supports it. The source lists spin/drop/panic possibilities but does not choose one. A production contract must define who observes failure and whether retry is safe; silent overwrites are unacceptable under the proposed contract (pp. 31–32; DEC-006).

Timer-driven flush requests must reach the owning writer rather than concurrently mutating its builders. Query cancellation releases its snapshot; it must not discard another query's input. Disk-write failures leave data uncommitted and must feed the bounded-backlog policy. Shutdown must specify intake stop, partial-batch publication, persistence drain and unresolved data reporting. These are proposed requirements, not implemented guarantees.

## Contract review evidence

| Concern | Closure evidence | Existing finding |
| --- | --- | --- |
| Exclusive mutation and publication | Restricted handles, state-machine review, concurrent-call schedules | VAL-012/013/019 |
| Retained queries and projections | Repeated/concurrent queries; reordered/nontrivial projections and schema checks | VAL-015/016 |
| Buffer reuse | Delayed-reader and persistence tests; allocation accounting | VAL-014/020/021 |
| Hybrid visibility and durability | Boundary/correction oracle; crash/restart injection | VAL-023/024 |
| Type fidelity | Arrow/Parquet/Flight/Python timestamp and numeric round trips | VAL-018/026 |

Related: [data flow](data-flow.md), [platform design](platform-design.md), [technical validation register](../00-governance/technical-validation.md), [decision register](../00-governance/decision-register.md).
