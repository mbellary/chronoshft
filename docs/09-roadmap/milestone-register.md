# Milestone registry

!!! info "Proposed registry · 2026-10-14"
    Milestones organize the [implementation roadmap](implementation-roadmap.md) and [MVP delivery plan](mvp.md). No implementation milestone is marked complete. Dates, named owners, approvals and executable evidence are unassigned.

## Status and evidence rules

Use **Proposed** for an unapproved plan, **Ready** after entry dependencies and approval records exist, **In progress** once delivery starts, **Blocked** with a recorded dependency or failed gate, and **Complete** only when all required exit evidence is reviewed and accepted. All entries below are Proposed; the dependency column identifies prerequisites rather than asserting an implementation has started.

Each evidence record must identify the milestone and requirement/decision IDs, implementation revision, artifact/harness/input versions, configuration, reviewer, date, outcome and remaining risks. Performance records also identify hardware, workload, timing endpoints, errors/losses and durability mode. A documentation build cannot substitute for executable evidence.

## Registry

| ID / increment | Proposed accountable role | Dependencies / decisions | Deliverable and exit gate | Status / evidence |
| --- | --- | --- | --- | --- |
| RM-01 — Scope baseline | Product + Domain | Research/Product drafts; DEC-001/002 | Approve financial boundary, full capability and venue/version matrix; explicitly disposition staged/deferred scope | Proposed; approval and named owner pending |
| RM-02 — Contract baseline | Architecture + Engineering | RM-01; DEC-003/004/006–010/013/014 | Version platform, schemas, financial semantics, ownership/reuse, visibility, overload, commit/recovery and client contracts; record required decisions | Proposed; reviewed baseline pending |
| RM-03 — Validation foundation | Validation + Performance | RM-02; DEC-005; DEC-011 if adopted profile requires it | Reproducible build, feed/domain oracles, concurrency/crash cases, workload definitions and requirement-linked harness inventory | Proposed; harness and acceptance review pending |
| RM-04 — Recent-data Slice A | Systems + Ingestion | RM-03; F01–F04/F12 | Selected feeds and concurrent hot SQL; verify safe lifetime, schema/order rules, bounded overload and query correctness; retain adopted hot-profile results | Proposed; implementation and results pending |
| RM-05 — Historical Slice B | Storage + Engineering | RM-04; F07–F10; DEC-008/009 | Parquet commit/recovery, tier handoff and late/correction plumbing; pass crash, retry and cross-tier oracle cases; retain adopted hybrid results | Proposed; implementation and results pending |
| RM-06 — Financial Slice C | Domain + Client Engineering | RM-05; F05/F06/F09–F11; DEC-013/014 | ASOF/bars/as-known workflow and supported Flight/dataframe clients; pass semantic and transport cases; retain adopted remote-profile results | Proposed; implementation and results pending |
| RM-07 — MVP release | Product + Operations + Security | RM-04/05/06; approved scope and required release checks | Complete [MVP release checklist](mvp.md#release-checklist); retain packaging, access, capacity/recovery evidence and named release approval | Proposed; release evidence and approval pending |

Accountable roles are planning proposals. Assign one named accountable owner and the necessary reviewers before moving an entry to Ready. Target and actual dates remain unset until an approved resourcing/dependency plan exists.

## Evidence destinations

| Gate family | Existing reference | Registry closure requirement |
| --- | --- | --- |
| Scope and decisions | [Decision register](../00-governance/decision-register.md), [Product readiness](../01-product/product-readiness.md) | Link actual approval/ADR records; source-established direction alone does not close a decision |
| Safety and correctness | [Engineering validation and traceability](../03-engineering/validation-and-traceability.md) | Link requirement-specific executable results and reviewed invariants; preserve VAL/GAP records until properly resolved |
| Performance | [Benchmark methodology](../06-benchmarking/benchmark-methodology.md), [acceptance criteria](../06-benchmarking/acceptance-criteria.md) | Link selected profiles, reproducible results and gate outcomes; avoid combining MVP/EPYC/burst targets |
| Release operations | [Operations readiness and traceability](../07-operations/readiness-and-traceability.md) | Link deployment, security, retention/capacity and recovery checks for approved scope |

## Updating a milestone

Record entry approval and owner before work starts. As evidence arrives, link it and report each gate outcome separately; a failed required gate prevents completion. For a blockage, record the concrete unmet dependency, accountable resolver and next review action. For a scope change, update Product requirements, the roadmap, affected validation cases and this registry together, preserving the prior decision history. Complete RM-07 only after release approval and all required evidence exist.

## Source traceability

RM-01/02 derive from the Phase 0 decision/gap handoff and Product/Architecture contracts. RM-03 is a proposed validation prerequisite. RM-04/05/06 retain the source hot → hybrid → financial/client sequence (BASE-001 pp. 26–28; RES-043–049), with the complete source MVP retained from pp. 16–17. RM-07 derives from Product MVP exit criteria and Benchmarking/Operations readiness. The seven registry IDs are new planning identifiers; they do not replace DEC, RES, PRD, VAL or GAP identifiers.
