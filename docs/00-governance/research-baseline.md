# Research baseline

!!! info "Phase 0 assessment · 2026-10-13"
    Extracted from CHRONOS-P0-001 v1.0. Assessment complete; engineering decisions pending review. Source intent, reviewer findings and approvals remain distinct.

| Control field | Value |
| --- | --- |
| Document ID | CHRONOS-P0-001 |
| Version / date | 1.0 / 2026-10-13 |
| Status | Assessment complete; engineering decisions pending review |
| Source | HFT - Research Notes.pdf; 42 physical PDF pages; 582,331 bytes |
| Source title | ChronosHFT: High-Frequency Trading TSDB Research & PRD |
| Source status | Draft / Vision Realisation; October 2026; Founding Team (p. 14) |
| Source identifier | BASE-001 |
| Source SHA-256 | `18b5ed26b6f4dfd8a83c452abf7cb81946f338078e66c85c71f48343631f37a2` |
| Authority | User-designated research baseline; source proposals are not automatically approved designs |
| Intended format | Markdown suitable for the future GitHub MkDocs repository |
| Scope | All 42 source pages; five assessment outputs; rules, baseline profiles and readiness gates |

!!! warning "Original PDF availability"
    The supplied assessment reports the BASE-001 metadata and review below. `research/HFT-Research-Notes.pdf` is an empty placeholder in this checkout, so the PDF hash, page references and reported inspection have not been independently verified here. Preserve this distinction until the matching original PDF is available.

## Assessment method and governance rules

1. **Preserve the baseline.** Retain BASE-001 unchanged, with its hash and page count. A revised source receives a new baseline ID and a documented delta.
2. **No silent requirements.** Extract source requirements as written in substance. Reviewer recommendations become gaps, validation actions or proposed decisions, never implicit product commitments.
3. **Separate existence from truth.** A source-stated requirement proves research intent, not feasibility or achievement. A code sample labelled production-grade is still unvalidated until its invariants and implementation are verified.
4. **Cite physical PDF pages.** `BASE-001 pp. X–Y` uses the PDF viewer page number, not repeated or inconsistent section numbering. Repeated claims share one extraction ID with all relevant pages.
5. **Paraphrase, preserve qualifiers.** Keep thresholds, units, hardware profiles, workloads and uncertainty. Do not convert “expected” metrics into measured results or “up to” into an unconditional sustained guarantee.
6. **Keep competing proposals.** Do not select the latest statement automatically. Distinct hardware, flush, query-cadence and roadmap profiles remain independently recorded until resolved.
7. **Use bounded validation.** Static inspection can identify missing protocols and direct code behavior. It cannot certify concurrency safety, runtime compatibility or latency. Record the evidence boundary explicitly.
8. **Separate approval from validation.** A technically supported fact is not a product approval; a product approval does not validate a benchmark.
9. **Keep bidirectional traceability.** Extraction → validation/gap/decision → target document → future requirement/design/test/evidence. Later artifacts must retain the originating extraction IDs.
10. **Version external evidence.** Record access date, upstream version/commit when known and scope. The source has no Cargo manifest or lockfile; live upstream documentation cannot establish compatibility of its samples.
11. **Do not import adjacent projects.** No architecture, contract scheme or runtime choice from other projects is adopted implicitly.
12. **Handle unknowns explicitly.** Use `not specified`, `unverified` or `decision pending`; do not fabricate owners, approvals, benchmark runs or dates.

## Classification vocabulary

| Dimension | Values and meaning |
| --- | --- |
| Extraction kind | Context; Objective; Requirement; Capability; Architecture; Algorithm; Prototype; Target; Commercial; Roadmap |
| Validation status | Validated: supported within stated evidence scope; Unverified: claim lacks evidence; Incorrect: stated claim contradicted by evidence or direct behavior; Incomplete: necessary contract/design is missing; Investigation: compatibility or feasibility needs dedicated work |
| Decision status | Source-established direction: clearly stated research intent; Proposed: technique/choice offered without approval; Unresolved: conflicting or unspecified choice |
| Disposition | Preserve; qualify; investigate; replace only through approved decision; defer scope |
| Priority | Critical: blocks safe/correct design; High: blocks measurable acceptance or production readiness; Medium: documentation, delivery or commercial uncertainty |

These identifiers are introduced for Phase 0, not claimed to exist in the PDF: `RES-NNN`, `VAL-NNN`, `GAP-NNN`, `CON-NNN`, `DEC-NNN`. IDs are stable and never reused. Register owners below are proposed roles, not assigned people. All open records have no committed due date.

## Work performed and evidence limits

The complete page-marked text was read, with local PDF extraction and visual inspection of the buffer-flip code on p. 38. The four prototype families were inspected statically. Selected primary-source checks were made for Arrow builder ownership, Flight transport and EPYC memory channels. No source code was compiled, no benchmark executed and no unsafe implementation certified. Broad competitive, market and protocol claims remain unverified.

## Baseline inventory

## Source-established directions

| Area | Preserved research direction | Source | Qualification |
| --- | --- | --- | --- |
| Product | Developer-first real-time financial TSDB | pp. 6–17 | Full trading engine boundary unresolved |
| MVP deployment | Single-node Rust binary with embedded DataFusion | pp. 13, 16, 26 | HA and fleet management appear as paid/future features |
| Foundation | Arrow memory; DataFusion query; Parquet persistence; Flight clients | pp. 2, 11–16 | Version and extension boundaries unspecified |
| Data plane | Ingest → hot batches → background cold persistence; unified query | pp. 8–9, 15–16 | Visibility and handoff semantics unresolved |
| User experience | SQL; Python/Polars/Pandas/Jupyter; financial primitives | pp. 6–7, 16–17, 27–28 | Exact API/dialect and conversion behavior unresolved |
| Time | Nanosecond-resolution timestamps | pp. 23–26 | Resolution does not establish ordering or clock accuracy |
| Deployment environment | Private VPC/on-premises/co-location; offline operation | pp. 7, 10, 17, 21–22 | OS, security and operational contracts missing |
| Commercial direction | Open core; paid fleet/HA/support; throughput/resource bands | pp. 17–18 | License and commercial assumptions unapproved |



## Evidence register and maintenance

| ID | Evidence | Scope and limit | Access date |
| --- | --- | --- | --- |
| BASE-001 | HFT - Research Notes.pdf, identified above | Authority for original research intent; no complete bibliography or reproducible results | 2026-10-13 |
| EXT-01 | [Apache Arrow Flight RPC specification](https://arrow.apache.org/docs/format/Flight.html) | Flight uses gRPC/IPC; does not measure Chronos copy/latency costs | 2026-10-13 |
| EXT-02 | [Apache Arrow PrimitiveBuilder implementation](https://arrow.apache.org/rust/src/arrow_array/builder/primitive_builder.rs.html) | Inspected finish implementation transfers values; upstream live source, no PDF version pin | 2026-10-13 |
| EXT-03 | [AMD EPYC 9354 specifications](https://www.amd.com/en/products/processors/server/epyc/4th-generation-9004-and-8004-series/amd-epyc-9354.html) | Twelve memory channels; does not validate assembled server performance or locality | 2026-10-13 |

External sources qualify technical assertions; they do not replace BASE-001 as the original research record. Future evidence must retain versioned source snapshots or commits where feasible. Requirements derived from proposed corrections require their own rationale and approval trail.

On revision, append a change record, keep IDs stable, record changed source/evidence links and reassess affected downstream artifacts. Do not delete rejected research from historical extraction; mark its disposition and link the superseding decision.

| Version | Date | Change |
| --- | --- | --- |
| 1.0 | 2026-10-13 | Initial complete Phase 0 assessment; 63 extraction records, 28 validation records, 22 gaps, 12 conflicts, 16 document destinations and 14 decisions |

## Source and related records

- [Unified Phase 0 assessment](../assets/research/ChronosHFT-Phase-0-Research-Baseline-Assessment.md) — preserved supplied document.
- [Research baseline](research-baseline.md) · [Assessment](research-assessment.md) · [Extraction matrix](research-extraction.md)
- [Technical validation](technical-validation.md) · [Assumptions & gaps](assumptions-and-gaps.md) · [Decision register](decision-register.md)
- [Performance profiles](performance-profiles.md) · [Traceability](traceability.md) · [Readiness & handoff](readiness-and-handoff.md)
