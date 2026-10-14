# Observability

!!! info "Research-derived operations draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages) and the Product, Architecture and Engineering contracts linked below. Source proposals and derived operational procedures remain distinct. No implemented control, approved SLA, recovery guarantee or measured performance is asserted.


## Purpose and evidence boundary

The source proposes simultaneous ingestion, queries and background persistence, with workload-specific performance targets (pp. 16–17, 21–23, 26–28). It does not specify a metrics backend, dashboard, alert thresholds or logging API. The following signals are derived from [component contracts](../02-architecture/component-contracts.md), [ingestion](../03-engineering/ingestion-engine.md), [memory lifecycle](../03-engineering/active-memory-lifecycle.md) and [validation](../03-engineering/validation-and-traceability.md); they are proposed operational instrumentation.

## Lifecycle and accounting

Report accepted, published/query-visible, durable and recyclable separately. Accepted ingestion is not durable acknowledgment. Track receive-to-visible age independently from query execution, query cadence and flush period (ENG-004/010/014; DEC-004/008).

Define mutually understood event categories and reconciliation windows: received, malformed/unsupported, duplicate, gap-affected, admitted, rejected/dropped, published and committed. Gap flags and duplicates may overlap other categories; they cannot simply be summed as disjoint losses. Reconcile by venue/partition/session and event identity; corrections are versions, not automatically duplicate events. Preserve last known sequence and affected ranges without treating equal timestamps as event identity (PRD-N06; ENG-002/007/012).

## Proposed signal inventory

| Domain | Signals | Investigation / contract |
| --- | --- | --- |
| Feed/adapter | Connection/reconnect, last receive, sequence gaps, malformed inputs, replay status and source clock quality | Distinguish idle market from stale/disconnected feed; DEC-002/007/014 |
| Writer/publication | Admission rate/outcomes, row/byte occupancy, oldest pending event, batch sizes and publish age | Full-ring/low-volume timer behavior; ENG-003/004/006 |
| Registry/memory | Hot retained bytes, outstanding snapshot/result leases, pool usage, process memory, allocations/deallocations and page faults | Delayed readers can retain memory after eviction; DEC-003/006 |
| Queries | Active/queued/rejected/cancelled queries, latency distribution by workload, errors, result bytes and completion | Separate server execution from client completion; ENG-008/013/014 |
| Persistence/catalog | Pending bytes/age, write/commit latency, failures/retries, committed coverage, disk free/reserves and incomplete outputs | Do not report write completion as durable commit unless selected model allows it; ENG-010/011 |
| Financial correctness | Gaps/degraded coverage, late/corrected versions, beyond-reorder-window events and handoff reconciliation failures | Preserve current/as-known semantics; ENG-007/009/011/012 |
| Flight/client | Streams, slow clients, bytes, disconnects, partial failures and released resources | Cancellation and retained-result limits; ENG-013 |
| Host | CPU per domain, IRQ placement, NUMA locality, memory bandwidth, disk/network saturation and clock drift | Measure shared-resource interference; DEC-005/011 |

## Proposed alert and investigation matrix

| Condition | Initial investigation and containment |
| --- | --- |
| Gap/disconnect or stale feed | Establish session/range and adapter status; invoke declared replay/resync policy; disclose incomplete coverage |
| Rising queue/publish age or retained memory | Identify writer, reader, client or flusher bottleneck; apply approved admission/concurrency limits before capacity exhaustion |
| Persistence failure or disk reserve reached | Keep incomplete files ineligible; bound backlog, surface uncommitted data and follow the selected intake policy |
| Tail-latency degradation | Identify exact workload/profile, load and measurement endpoint; inspect CPU/IRQ, memory, persistence and stream contention |
| Handoff/correction mismatch | Preserve evidence and affected versions; restrict affected results under the declared consistency policy |
| Cancellation resource leak | Identify retained stream/snapshot ownership; enforce the selected timeout/cancellation contract |

Numeric thresholds, observation windows, escalation roles and severity are pending review. Derive them from approved budgets and recovery requirements, including time-to-exhaustion under measured load. Source P99 < 1 ms and EPYC P99 < 850 µs are unverified workload targets, not universal alert thresholds or SLAs.

## Logs and diagnostic evidence

Proposed structured fields: build/config/schema identity, component, request/stream/batch/file identity, venue/partition/session, sequence coverage, lifecycle transition, outcome and error. Bound log buffering/rate/retention; expensive formatting or synchronous logging on the writer can disturb latency. Avoid secrets and unbounded per-event logs. Capture sampled diagnostics and counters without losing loss accounting.

Attach hardware, effective configuration, source fixtures, benchmark harness, latency endpoints/distributions, errors/losses, memory/backlog, freshness and commit mode to incidents and performance runs. Keep instrument labels bounded; detailed sequences belong in controlled diagnostic records. See [capacity planning](capacity-planning.md), [recovery](recovery.md) and [Engineering validation](../03-engineering/validation-and-traceability.md). Backend choice and instrumentation overhead remain unvalidated.
