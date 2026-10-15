# Validation registry and baseline migration

!!! info "Stage 3 migration · 2026-10-15"
    All 28 historical validation records were imported into YAML with empty downstream registries. Stage 4 added experiment definitions; Stage 5 adds the [test catalog](../10-validation-lab/test-register.md) and revision-pinned applicability links. Executions, evidence and reviewer decisions remain pending.

## Authority boundary

The [research register](technical-validation.md#research-assessment-baseline) remains authoritative for original findings, research statuses, source references and closure requirements. The supplied Phase 0 assessment remains unchanged. The [framework contract](validation-framework.md) defines how engineering validation extends that baseline.

`validation/register/validations/VAL-NNN.yaml` is authoritative for current engineering metadata: revision, objective, classification, priority, coverage, lifecycle and current decision reference. Its `origin` fields are a checked copy of the historical assessment. The [Technical Validation Lab](../10-validation-lab/index.md) generates its coverage, registers, experiment pages and results from the structured records. Edit YAML and regenerate those views. The [methodology](../10-validation-lab/methodology.md) is authored Markdown and retains the workflow and authority boundaries.

The historical `origin.required_closure_evidence` stays fixed. Current `required_evidence` starts with that text and may evolve through an explicit engineering revision. Updated requirements do not automatically supersede architecture or product decisions.

## Migration contents

| Data | Treatment |
| --- | --- |
| VAL-001–VAL-028 | Preserved exactly; one YAML file per record |
| RES references and research statuses | Preserved verbatim, including abbreviated ranges and status qualifiers |
| Findings and closure evidence | Preserved verbatim in origin fields; closure evidence also initializes engineering requirements |
| Provenance | CHRONOS-P0-001 v1.0, dated 2026-10-13, linked to the historical register |
| Titles and objectives | Provisional engineering descriptions derived from findings and closure requirements |
| Category and priority | `null`; the historical table assigns neither |
| Initial state | Revision 1, lifecycle `identified`, coverage `unmapped`; recorded as a migration event on 2026-10-15 |
| Decisions | No current or prior decision; technical disposition is pending |

This table describes the initial import. Subsequent progress belongs in the structured records. Existing role-only and bounded-arithmetic research findings do not become engineering acceptance.

## Schemas and layout

`validation/schemas/` contains JSON Schema Draft 2020-12 definitions for validation, experiment, test, execution, evidence and decision records, plus the baseline manifest. Record schemas require `schema_version: 1`, stable identities and explicit fields, reject unknown fields, and distinguish unassigned values from conclusions.

The layout reserves `validation/register/experiments/`, `validation/register/tests/`, `validation/executions/`, `validation/evidence-manifests/` and `validation/decisions/`. Large artifacts will be retained separately from lightweight manifests under the existing [evidence bundle plan](../06-benchmarking/evidence-and-traceability.md#proposed-evidence-bundle).

## Checks and reproduction

`scripts/validate_register.py` checks:

- Schemas, required fields, states, IDs, filenames and duplicate YAML keys.
- Preservation of all 28 origin records against the historical Markdown table.
- Digests for the original table rows and supplied assessment, pinned in `validation/baseline.yaml`.
- Initial closure evidence, revision histories and legal recorded lifecycle transitions.
- Prior definition snapshots, identities and retained provenance under `validation/revisions/` when revisions advance.
- Local references, reciprocal planning relationships and hypothesis ownership when introduced.
- Mandatory hypothesis/method/applicability coverage with `--require-tests`, reciprocal validation/test obligations and execution binding before experiment advancement.
- Basic consistency of execution outcomes and validation decision references.

Quote dates to keep YAML compatible with JSON Schema. Follow the [identity and revision rules](validation-framework.md#identity-and-revision-rules) for changes. A changed research finding requires a dated amendment with supporting evidence; silently editing both copies is not valid preservation.

From the repository root:

```bash
uv sync --locked --extra docs --extra validation
uv run --locked --extra docs --extra validation python scripts/check_documentation.py
```

The shared gate now also resolves pinned historical definitions, protects immutable ledgers against the Git event baseline, verifies retained artifact content and checks recorded decision coverage. The [execution/review workflow](../10-validation-lab/execution-and-evidence.md) describes those tools and their limits. Reviewer attestation and domain judgment remain explicit; a successful schema/hash check cannot grant approval. Git history protects previously committed baseline manifests and original source content from silent replacement.

`scripts/generate_validation_docs.py` generates all Lab projections and updates only the marked Lab section in `mkdocs.yml`. `--check` verifies content and navigation without writing. Unregistered experiment pages are rejected. The old `scripts/generate_validation_coverage.py` command invokes the same generator. Earlier governance coverage/test URLs preserve their anchors and link to the canonical Lab pages; they hold no independent status tables.

## Next stage

Stage 8 adds pinned capture, manual import/recovery, retained evidence, Git-history protection and explicit decision verification/registration. It changes no existing validation/test revision, lifecycle state or decision and registers no real run. Stage 9 implements EXP-001, binds its tests and records its actual observations within their limited scope.
