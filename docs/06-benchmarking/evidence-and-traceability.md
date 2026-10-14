# Evidence and traceability

!!! info "Validation evidence plan · 2026-10-14"
    This page maps extracted research and proposed validation procedures to existing requirements and findings. No raw benchmark results, approved decisions or closed validation records are supplied by this update.

## Research extraction map

Page numbers below are physical pages in [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001). Existing RES/VAL identifiers come from the governance records; proposed test procedures do not become source claims.

| Source pages / existing records | Benchmarking destination | Product / architecture dependency |
| --- | --- | --- |
| 17; RES-027/029/032 | MVP and DX workloads; acceptance selection | PRD-N01/N02, F11; product boundary and client transport |
| 21–23; RES-034–039 | Hardware manifest, reference ingestion/burst/tails/egress | Deployment topology; DEC-005/011; VAL-008/009 |
| 26–28; RES-045/047/048 | Stage workloads, allocation audit, hybrid and remote dataframe gates | Scope & MVP; data flow; PRD-F03/F07/F11 |
| 29–32; RES-051–053 | Full-ring, wraparound, allocation and lifetime tests | Active memory; PRD-N03/N06/N08; VAL-012–014 |
| 33–36; RES-054–057 | Non-destructive scans, projections and plan/schema verification | Query/storage integration; VAL-015–018 |
| 36–42; RES-058–062 | Timer ownership, delayed-reader reuse and signaling measurements | Component contracts; VAL-019–022; DEC-003/004 |
| 9, 16, 27–28; RES-018/019/030/046/049 | Commit/restart and hot/cold correction oracles | PRD-F07–F10/N07/N08; VAL-023/024; DEC-008/009 |
| 6–10, 17, 23–25, 28 | Financial and timestamp/client fidelity tests | PRD-F05/F06/F11/F12; VAL-004/010/011/018/026/027 |
| Product benchmark document, sections 1–5 | Additional one-second burst, tails, local egress and storage profiles | User-proposed targets; distinct from PDF profiles |

Use the [research extraction matrix](../00-governance/research-extraction.md), [technical validation](../00-governance/technical-validation.md) and [product traceability](../01-product/product-traceability.md) for authoritative existing record descriptions. The [Engineering validation map](../03-engineering/validation-and-traceability.md) provides complementary ENG coverage.

## Proposed evidence bundle

Retain a versioned bundle per run with the following contents; no bundle is claimed to exist yet.

| Artifact | Required content |
| --- | --- |
| Manifest | Run ID/date, profile/requirement IDs, commit/dirty state, dependency/build versions and hardware/configuration |
| Inputs | Fixture/generator version, seed/hash, schema, arrival schedule, exact SQL, oracle and expected identities/results |
| Measurements | Raw samples/histograms, percentile resolution, warmup/duration/trials, timing endpoints, errors/timeouts and profiler overhead |
| Accounting | Offered/accepted/rejected/duplicate/gap counts, visible/durable cutoffs, queue occupancy/drain, RSS/allocations/page faults and bytes |
| Correctness/safety | Oracle comparisons, lifetime argument, applicable model/tool logs and failure-injection schedules |
| Review | Pass/fail/blocked/not-applicable disposition, deviations, limitations, named reviewer/date and linked decisions |

Preserve unsuccessful trials and exact reproduction commands. A summarized percentile without workload/manifest/raw evidence cannot close a performance finding. Record deviations from the source target and product target separately.

## Open decisions and closure artifacts

| Existing decision / gap | Required review artifact |
| --- | --- |
| DEC-003; GAP-002/003/012 | Ownership alternative, restricted handles, lifetime/reuse argument and complete memory budget |
| DEC-004/006; GAP-004 | Independent publication/freshness/flush settings and overload/loss-accounting contract |
| DEC-005/011; GAP-013 | Approved workload/timing contract and actual hardware/locality/interference results |
| DEC-007/014; GAP-006/008/009/014 | Schema, sequence, timestamp and financial operator oracle definitions |
| DEC-008/009; GAP-005/007/010 | Acknowledgment/failure/commit/version model and crash/handoff tests |
| DEC-013; GAP-011/015 | Pinned supported clients, conversion/types, cancellation and access contract |
| GAP-022 | Requirement-linked executable suite with applicable safety/concurrency tooling |

All remain pending here. Proposed reviewing roles and gate order are documented in [acceptance criteria](acceptance-criteria.md); this page assigns no named owners and introduces no approved DEC/VAL IDs.

## Documentation verification

Run the repository's strict build after content/navigation changes:

```bash
uv run --locked --extra docs mkdocs build --strict
```

Check local links and source/profile identifiers, and ensure all Benchmarking pages are reachable from navigation. A successful build establishes documentation integration only; it is not runtime performance, implementation safety, product approval or site publication evidence.
