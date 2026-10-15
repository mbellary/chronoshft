# Technical validation register

!!! info "Phase 0 assessment · 2026-10-13"
    Extracted from CHRONOS-P0-001 v1.0. Assessment complete; engineering decisions pending review. Source intent, reviewer findings and approvals remain distinct.

## Register authority and preservation

This register is the authoritative record of the Phase 0 research findings for VAL-001–VAL-028. Each record retains its identity, related RES references, original assessment status, finding, evidence boundary and required closure evidence. The assessment date above applies to those findings; later engineering work does not rewrite that historical assessment.

New experiments and evidence must reference the existing VAL IDs. Preserve the original assessment when recording a subsequent result, including when an implementation corrects an identified issue. If a research finding needs correction, record a dated amendment with its reason and supporting evidence rather than silently replacing the original finding. Keep the [supplied Phase 0 assessment](../assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md) unchanged as the source baseline.

## Research assessment baseline

The status column below describes the original research assessment, not current implementation readiness or validation acceptance. In particular, the bounded findings for VAL-002 and VAL-010 do not establish that the ChronosHFT integration or timestamp implementation has passed validation.

| ID / related extraction | Research assessment status | Finding and evidence boundary | Required closure evidence |
| --- | --- | --- | --- |
| VAL-001 / RES-001 | Unverified | Fast assembly of primitives does not establish production readiness | Reliability/security/correctness evidence; qualify commercial wording |
| VAL-002 / RES-002 | Validated (role only) | FDAP roles are consistent with the source and Flight/Arrow primary documentation [EXT-01/02]; Chronos integration untested | Pin compatible versions and integration tests |
| VAL-003 / RES-006 | Unverified | Zero-CPU and 10–20× compression claims lack workloads; Arrow/Parquet encoding integration absent | Losslessness tests; compression/CPU benchmarks and format design |
| VAL-004 / RES-014 | Incorrect (absolute claim) | Flight uses gRPC and Arrow IPC [EXT-01]; no-serialization/no-network-parsing guarantee is too broad | Boundary-specific copy/serialization measurements and client matrix |
| VAL-005 / RES-020 | Incomplete | No tracing GC does not bound allocator, destruction, paging, scheduling or I/O latency | Define bounded path and allocation/deallocation/page-fault budgets |
| VAL-006 / RES-027,029,036–039,045,047–048 | Unverified | No measured runs, harness, hardware record or result artifacts | Reproducible workload-specific evidence |
| VAL-007 / RES-033 | Unverified | Categorical competitor shortcomings and example latency/loading comparisons lack reproducible evidence | Versioned feature audit and fair benchmark; no universal claims |
| VAL-008 / RES-034 | Incorrect (population claim) | Six DIMMs are described as fully populated; EPYC 9354 has twelve memory channels [EXT-03] | Corrected proposed BOM and motherboard population plan |
| VAL-009 / RES-034–035 | Incomplete | Single socket/pinning alone do not prove locality or jitter isolation; OS and flusher share cores | Record BIOS/NUMA/SMT/IRQ/affinity and measure interference |
| VAL-010 / RES-040,042 | Validated (bounded arithmetic) | Signed 64-bit ns is eight bytes and roughly ±292 years around epoch; unit precision is not clock accuracy | Boundary/overflow/timezone round-trip tests |
| VAL-011 / RES-041 | Incorrect (ordering inference) | Equal ns timestamps still possible; timestamps alone cannot guarantee feed event ordering | Sequence/partition/tie-break rules and replay tests |
| VAL-012 / RES-051,053 | Incomplete | Writer appends before full-ring check; failed rotate returns and later appends continue in the same slot; capacity bound unenforced | Full-queue policy before writes, overflow tests and protocol proof |
| VAL-013 / RES-051 | Incomplete (safety) | Public `&self` append/poll methods plus unsafe Sync do not enforce one producer/consumer; concurrent callers can alias mutable slots | Restricted handles and documented exclusion/publication invariants |
| VAL-014 / RES-052 | Incorrect for inspected upstream behavior | PrimitiveBuilder finish transfers its values using `mem::take`; capacity is not retained in that builder [EXT-02]. Batch drops do not invoke the source's reserve loop | Version-pinned allocation test and explicit pool/reclamation design |
| VAL-015 / RES-054 | Incorrect for persistent-table semantics | scan consumes the queue; later/concurrent scans may see different/disjoint batches. It is not a retained snapshot registry | Repeatable non-destructive scans and snapshot boundary tests |
| VAL-016 / RES-054 | Investigation | Projection is applied to each batch and passed again to MemoryExec with original schema; risks depend on API version | Nontrivial/reordered projections and schema consistency tests |
| VAL-017 / RES-055,060 | Incomplete | Example calls `schema()` and `row_count()` not defined in shown structs; no manifest/version pins | Compile complete examples against a pinned dependency set |
| VAL-018 / RES-040,051 | Incomplete | Code uses Int64 schema rather than Arrow Timestamp(ns); timezone/epoch semantics not encoded | Approved schema and Arrow/Parquet/Flight round-trip tests |
| VAL-019 / RES-058 | Incomplete (safety) | Atomic index flip does not enforce exclusive append/flush access or track safe reuse. No reader lease/retirement protocol is shown | Ownership state machine; concurrent-call tests; memory model review |
| VAL-020 / RES-058,056 | Incorrect (guarantee) | Moving builder storage into immutable arrays differs from querying shared mutable buffers. Either allocation or safe reclamation must be addressed; flip alone proves neither | Explicit transfer/reuse model with delayed readers |
| VAL-021 / RES-060–061 | Incorrect (unconditional guarantee) | Disruptor event completion is not necessarily DataFusion query completion if the callback hands off a batch | Query lifetime tied to buffer retirement; slow-reader tests |
| VAL-022 / RES-062 | Unverified | Busy-spin latency/alignment claims depend on crate version and platform | Verify source layout; measured signaling and CPU cost |
| VAL-023 / RES-019,030,046,049 | Incomplete | Hybrid visibility, late/corrected data and historical knowledge policies absent | Model and oracle tests covering handoff/corrections |
| VAL-024 / RES-018 | Incomplete | Parquet format/background writes alone do not establish crash-consistent durable acknowledgment | Failure model; commit/catalog protocol and restart tests |
| VAL-025 / RES-025,035 | Investigation | DPDK packet ingress and WebSocket application decoding are distinct integration problems | Adapter-specific transport, framing, TLS and allocation design |
| VAL-026 / RES-028 | Incomplete | “ANSI SQL” and financial functions lack a supported subset and precise semantics | Dialect/version and function conformance suite |
| VAL-027 / RES-021 | Incomplete | Order-book reconstruction lacks venue event/state rules and recovery policy | Venue-specific reference traces and deterministic oracle |
| VAL-028 / RES-063 | Incomplete | Numeric citations cannot be resolved to references from the PDF | Recover bibliography or mark each affected assertion unsupported |

## Engineering progress and validation decisions

Track subsequent work separately from the research assessment:

| Record dimension | Purpose | Relationship to this baseline |
| --- | --- | --- |
| Research assessment | Preserve what Phase 0 established and the limits of that finding | The 28 records above remain the authoritative historical findings |
| Engineering progress | Record preparation, implementation, execution and evidence review for each validation | Reference the VAL ID without changing its research assessment status |
| Validation decision | Record what reviewed evidence establishes, with its scope, limitations and approval | Link the VAL ID, supporting evidence and applicable implementation revision |

The [Stage 3 registry migration](validation-registry.md) initializes each validation's engineering record at revision 1 with lifecycle `identified`, coverage `unmapped` and no decision. Current engineering metadata is maintained in `validation/register/validations/`; the table above preserves the historical assessment. No new executions, evidence, reviewer approvals or closures are recorded by that migration. Existing research statuses remain unchanged; engineering acceptance remains pending.

The [coverage plan](../10-validation-lab/validation-coverage.md) records the Stage 4 transition to `planned`. Stage 5 adds the [test specifications](../10-validation-lab/test-register.md), validation revision 3 and experiment revision 2, preserving prior definitions as snapshots. Experiments remain `proposed` and execution bindings remain unbound. Progress updates must not imply technical acceptance. A passing test supports only the behavior and environment it exercised. A validation decision must identify its evidence and review boundary, and any related architecture approval must remain explicit in the [decision register](decision-register.md). Documentation completion alone does not close a validation.

The [validation framework contract](validation-framework.md) defines identities, relationships, lifecycle states, evidence and decision rules. The [registry migration](validation-registry.md#authority-boundary) defines the checked authority boundary. The [Technical Validation Lab](../10-validation-lab/index.md) presents generated coverage, test specifications and separate execution/evidence/results views. Stage 8 adds the [capture and review tools](../10-validation-lab/execution-and-evidence.md) without registering a real run or advancing a validation. Generated views retain access to this historical baseline and distinguish it from current engineering progress and decisions.

## Critical prototype interpretation

The SPSC design must not be reported as intrinsically racy under every possible restricted use: its sequence pattern suggests an intended single producer/consumer discipline. The problem is that this discipline is not enforced by the exposed API and its capacity/full-ring behavior is incomplete. Similarly, `finish()` can detach array storage, so the double-buffer sample does not prove that a query reads builder memory being overwritten. The justified findings are missing caller exclusion, unsupported no-allocation/reuse claims, and absent lifetime-based reclamation. A stronger data-race conclusion requires a concrete supported concurrent-call schedule.

The TableProvider issue is direct: polling advances the shared read sequence. Query planning therefore consumes ingestion output rather than taking a stable view of a retained table. The Disruptor integration's callback only prints rows; it does not implement the claimed DataFusion registry or query-completion barrier.

## Evidence references

`EXT-01`–`EXT-03` refer to the [evidence register](research-baseline.md#evidence-register-and-maintenance). These are the supplied assessment's dated checks, not new validation performed during publication.

## Source and related records

- [Unified Phase 0 assessment](../assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md) — preserved supplied document.
- [Research baseline](research-baseline.md) · [Assessment](research-assessment.md) · [Extraction matrix](research-extraction.md)
- [Technical validation](technical-validation.md) · [Assumptions & gaps](assumptions-and-gaps.md) · [Decision register](decision-register.md)
- [Performance profiles](performance-profiles.md) · [Traceability](traceability.md) · [Readiness & handoff](readiness-and-handoff.md)
