# Validation registries

Stage 3 introduces schema-version 1 records and imports VAL-001–VAL-028. See `docs/00-governance/validation-framework.md` for the framework and `docs/00-governance/validation-registry.md` for migration and authority rules.

| Location | Content |
| --- | --- |
| `baseline.yaml` | Assessment identity, original record IDs and integrity digests |
| `schemas/` | JSON Schema Draft 2020-12 definitions for six record types and the baseline |
| `register/validations/` | Historical provenance plus current engineering metadata |
| `register/experiments/` | Stage 4 experiment definitions and reciprocal validation links |
| `revisions/{validations,experiments,tests}/<ID>/rN.yaml` | Retained prior definitions before revisions advance |
| `register/tests/` | Stage 5 test specifications, applicability and execution bindings |
| `execution-plans/` | Committed capture plans, beginning with EXP-001.yaml |
| `executions/` | Append-only RUN records with exact source/definition pins |
| `evidence-manifests/` | Append-only EVD manifests with retained artifact digests |
| `decisions/` | Append-only VDEC decisions; no real approvals registered yet |
| `artifacts/sha256/` | Small content-addressed retained assets; large artifacts use ignored `evidence/<RUN-ID>/` bundles |

Record filenames are `<ID>.yaml`. Quote ISO dates (`'2026-10-15'`) and UTC timestamps (`'2026-10-15T12:00:00Z'`) so YAML values remain JSON-compatible strings. Unknown fields, duplicate keys and unsupported schema versions are rejected. Unassigned values use `null`. Disposition is derived from the current decision reference, or pending when none exists; it is not an editable field.

The initial import records revision 1, lifecycle `identified`, coverage `unmapped`, unassigned category/priority, and no decisions. Titles and objectives are provisional engineering metadata derived from closure requirements, not additional research findings. Source-reference strings are preserved verbatim. Stage 4 records validation revision 2, lifecycle `planned` and mandatory experiment links. Stage 5 records validation revision 3, experiment revision 2 and 90 test definitions at revision 1. Prior definitions are archived; lifecycles remain planned/proposed.

The experiment registry contains EXP-001–EXP-003 and EXP-005–EXP-030: 29 investigations. The proposal left EXP-004 unassigned; no extra experiment is invented to match its stated count of 30. EXP-003 explicitly contributes publication-protocol evidence to VAL-012 and ownership evidence to VAL-013.

From the repository root:

```bash
uv sync --locked --extra docs --extra validation
uv run --locked --extra docs --extra validation python scripts/generate_validation_docs.py
uv run --locked --extra docs --extra validation python scripts/check_documentation.py
```

The checker locates the repository relative to its script; `--root` selects an isolated repository fixture for tests. The historical Markdown table remains authoritative for original research; YAML is authoritative for current engineering metadata. Do not maintain another current-status table. Original closure evidence is checked verbatim at revision 1. Later engineering requirements may evolve through explicit revisions while `origin` remains unchanged.

Before increasing a VAL, EXP or TST revision, retain its full previous definition under `validation/revisions/{validations,experiments,tests}/<ID>/r<revision>.yaml`. The checker requires schema-valid snapshots for prior revisions and checks their identity, retained revision history and research provenance. This preserves definitions during uncommitted staged work as well as in Git history; archive files are historical definitions, not additional current records.

`--require-coverage` checks that every validation has a mandatory investigation. `--require-tests` additionally checks mandatory hypothesis, method and validation applicability coverage. Tests declare a revision-pinned owning experiment, scoped hypotheses and reciprocal applicable validations. Each validation's mandatory test flag must agree with its test specification.

Tests start with `execution_binding.state: unbound`, null implementation path and null entrypoint. A bound method needs a concrete command/review entrypoint; automated methods also need a source path. Unbound mandatory methods prevent experiment advancement. Source existence, executable behavior and full prerequisite fulfillment remain execution-readiness checks; bindings alone never imply a run or a pass. Approved budgets, policies and fixture/environment values must be fixed before contract-dependent execution.

`scripts/generate_validation_docs.py` validates full test coverage and generates the Lab overview, coverage, registers, environment view, results and per-experiment pages under `docs/10-validation-lab/`. It updates only the marked Lab navigation section in `mkdocs.yml`. The methodology page is authored separately. `--check` rejects missing/stale pages, navigation drift and unregistered experiment pages without writing.

`scripts/generate_validation_coverage.py` remains a compatibility entrypoint for the full generator. The earlier governance coverage/test URLs preserve their anchors and link to canonical Lab pages. No independent manual status tables are retained. Displayed runs, manifests and decisions are projections of records, not artifact verification or inferred approval. Environment profile categories are preparation requirements; actual environments come from RUN records.

The checker compares imported origin fields to Markdown and verifies digests for the original rows and supplied assessment. Changing baseline digests requires an explicit reviewed migration or amendment. Digests detect drift relative to the committed manifest; they do not prevent deliberate edits to that manifest.

The shared documentation gate runs full registry coverage, Git-baseline checks, execution/artifact/decision verification, generated views (`--check`), regressions, strict MkDocs build and rendered local links. It stops at the first failure. CI never regenerates content to hide drift. Both workflow files, schemas, registries, scripts and tests are covered by the Documentation trigger filters.

Stage 8 adds these tools:

| Tool | Responsibility |
| --- | --- |
| `capture_execution.py` | Capture exact bound automated commands, toolchain/environment, outcomes, logs and declared artifacts against committed source |
| `register_execution.py` | Verify/import manual observations or complete identical staged publication with `--resume` |
| `verify_evidence.py` | Resolve pinned definitions and verify actual artifact content, outcomes, input/lock hashes and decision evidence |
| `check_validation_history.py` | Protect committed ledgers/snapshots/artifacts and require explicit definition revisions against the trusted Git baseline |
| `review_validation.py` | Prepare an unapproved package; check a supplied human decision; register only with explicit `--register-reviewed` |

See `docs/10-validation-lab/execution-and-evidence.md` for plan fields, retention policies, mirror mapping, CI capture, recovery and review steps. No schema/check result authenticates a reviewer or supplies domain judgment. Positive decisions need complete declared coverage, a concrete scope and verified supporting artifacts; current-decision selection/lifecycle changes remain explicit. Stage 9 binds EXP-001 to the isolated Rust prototype; Stage 10 investigates EXP-002 overload behavior. Documentation regressions remain isolated fixtures and create no actual project results.
