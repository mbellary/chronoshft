# Automation and CI gates

The Documentation workflow in `.github/workflows/documentation.yml` runs the same fail-fast gate used locally. It checks registry and documentation consistency before the existing publication steps. These are repository regression checks; they do not execute the planned TST experiments or approve a validation.

## Gate sequence

| Order | Check | Required result |
| --- | --- | --- |
| 1 | Registry schemas and relationships | All records conform; historical findings and snapshots are preserved; revision references, recorded lifecycle histories and mandatory test coverage are consistent |
| 2 | Git history and immutable ledgers | Definitions preserve revision/provenance histories; committed runs, evidence, decisions, snapshots and artifacts remain append-only against the event baseline |
| 3 | Execution, artifact and decision verification | Historical definitions resolve; artifact sizes/hashes and source/input/lockfile references match; recorded decisions satisfy their evidence and applicability checks |
| 4 | Generated Lab pages and navigation | Every generated file and the marked navigation section match YAML; no unregistered experiment page remains |
| 5 | Regression checks | Registry, runtime, review, generator, link-checker and CI integration regressions pass |
| 6 | Strict MkDocs build | All documentation builds under the locked environment without a strict-build failure |
| 7 | Rendered local links and anchors | Local hyperlink targets exist and HTML fragments resolve in the built site, including compatibility pages |

`scripts/check_documentation.py` runs these checks in order and stops at the first failure. The generated-content check uses `--check`; CI does not repair stale committed documentation. A failed gate prevents the Pages artifact upload, and deployment depends on the successful build job.

The link checker examines rendered local hyperlinks. It does not fetch external URLs. The separate evidence gate verifies retained content or an explicit local mirror; inaccessible content fails. Git history is fetched for the trusted event-baseline comparison. The [execution workflow](execution-and-evidence.md) describes capture, recovery, retention and review boundaries.

## Workflow triggers and publication

Relevant pushes to `main`, pull requests targeting `main` and manual dispatch run the workflow. Path filters cover `docs/`, `research/`, `validation/`, `scripts/`, `tests/`, MkDocs configuration, dependency/lock files, the Python version, README and both documentation/capture workflows. A schema-only, generator-only or test-only change therefore runs the complete gate.

The workflow installs the locked `docs` and `validation` dependency extras. It retains the existing `Validate and build documentation` job name and read-only build permissions. The build has a 15-minute timeout. Pull requests run checks; Pages upload/deployment remain restricted to eligible `main` runs. The deployment job retains its existing scoped Pages permissions and environment.

These workflow checks do not configure repository branch-protection settings. Local success demonstrates the configured gate behavior; a hosted CI result exists only after GitHub runs the committed workflow.

## Run locally

Install the locked environment and run the complete gate:

```bash
uv sync --locked --extra docs --extra validation
uv run --locked --extra docs --extra validation python scripts/check_documentation.py
```

After editing authoritative YAML, regenerate and review the documentation before running the gate:

```bash
uv run --locked --extra docs --extra validation python scripts/generate_validation_docs.py
uv run --locked --extra docs --extra validation python scripts/check_documentation.py
```

The gate writes the normal ignored `site/` build output. It does not rewrite YAML, generated Markdown or MkDocs navigation. Both scripts find their repository relative to their own location; the runner executes each check from that repository root.

## Resolve failures

- **Registry:** correct the record, applicable references or revision snapshot. Preserve original research findings and add explicit revisions for definition changes.
- **History:** retain prior content and append a new identity/revision as required; fetch the trusted comparison commit if it is unavailable.
- **Evidence:** restore the exact retained content or supply `VALIDATION_ARTIFACT_MAP` for a local mirror. Corrected artifacts require new EVD identities; a hash cannot be edited to hide corruption.
- **Generated content:** regenerate the Lab from YAML, review the Markdown/navigation changes and rerun the gate. Do not manually patch a generated status table.
- **Regression:** inspect the reported assertion or error, correct the behavior and rerun the gate.
- **Strict build:** correct the source document, configuration or missing Markdown destination.
- **Rendered link:** correct the local file path or target anchor. A successful Markdown build alone cannot substitute for a missing rendered destination.

## Evidence and acceptance boundary

Passing this workflow establishes documentation and registry integration. The [execution register](execution-register.md), [evidence register](evidence-register.md) and [validation results](validation-results.md) continue to derive their content only from registered records. No RUN, EVD or VDEC record is created by a CI documentation run.

The separate manually dispatched capture workflow retains observations, including normal failures, without pushing records or approving validations. Shared CI refuses performance-class acceptance cases. Performance acceptance still requires its declared controlled environment, workload and retained observations. Reviewer decisions and architecture approvals remain explicit under the [framework contract](../00-governance/validation-framework.md). Stage 9 supplies the bound EXP-001 capture plan; its sequential observations require separate review. Stage 10 investigates EXP-002 overload behavior.
