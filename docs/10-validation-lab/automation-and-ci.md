# Automation and CI gates

The Documentation workflow in `.github/workflows/documentation.yml` runs the same fail-fast gate used locally. It checks registry and documentation consistency before the existing publication steps. These are repository regression checks; they do not execute the planned TST experiments or approve a validation.

## Gate sequence

| Order | Check | Required result |
| --- | --- | --- |
| 1 | Registry schemas and relationships | All records conform; historical findings and snapshots are preserved; revision references, recorded lifecycle histories and mandatory test coverage are consistent |
| 2 | Git history and immutable ledgers | Definitions preserve revision/provenance histories; committed runs, evidence, decisions, snapshots and artifacts remain append-only against the event baseline |
| 3 | Execution, artifact and decision verification | Historical definitions resolve; artifact sizes/hashes and source/input/lockfile references match; recorded decisions satisfy their evidence and applicability checks |
| 4 | Generated Lab pages and navigation | Every generated file and the marked navigation section match YAML; no unregistered experiment page remains |
| 5 | Native Cargo regression checks | Registry, runtime, review, generator, link-checker and CI integration regressions pass |
| 6 | Strict MkDocs build | All documentation builds under the locked environment without a strict-build failure |
| 7 | Rendered local links and anchors | Local hyperlink targets exist and HTML fragments resolve in the built site, including compatibility pages |

`cargo lab check` runs these checks in order and stops at the first failure. Steps 1–4 and 7 invoke the native CLI against the selected root; step 5 runs `cargo test --offline --locked --workspace`; step 6 invokes `uv run --locked --extra docs python -m mkdocs build --strict`. The generated-content check uses `--check`; CI does not repair stale committed documentation. A failed gate prevents the Pages artifact upload, and deployment depends on the successful build job.

The link checker examines rendered local hyperlinks. It does not fetch external URLs. The separate evidence gate verifies retained content or an explicit local mirror; inaccessible content fails. Git history is fetched for the trusted event-baseline comparison. The [execution workflow](execution-and-evidence.md) describes capture, recovery, retention and review boundaries.

## Workflow triggers and publication

Relevant pushes to `main`, pull requests targeting `main` and manual dispatch run the workflow. Path filters cover `docs/`, `research/`, `experiments/`, `crates/`, `.cargo/`, root Cargo manifests/lockfiles, `validation/`, `tests/`, MkDocs configuration, dependency/lock files, the Python version, README and both documentation/capture workflows. A schema-only, generator-only or test-only change therefore runs the complete gate.

The workflow installs the locked `docs` dependency extra, prepares stable Rust and fetches locked Cargo dependencies, and checks Rust formatting/lints. Its required native gate then runs the sequence above. The native gate must pass before Pages upload; the Python comparison suite was retired after parity checks. It retains the existing `Validate and build documentation` job name and read-only build permissions. The build has a 15-minute timeout. Pull requests run checks; Pages upload/deployment remain restricted to eligible `main` runs. The deployment job retains its existing scoped Pages permissions and environment.

These workflow checks do not configure repository branch-protection settings. Local success demonstrates the configured gate behavior; a hosted CI result exists only after GitHub runs the committed workflow.

## Run locally

Install the locked environments and run the complete gate. Rust/Cargo and `uv` must be available on PATH:

```bash
uv sync --locked --extra docs
cargo fetch --locked
cargo lab check
```

After editing authoritative YAML, regenerate and review the documentation before running the gate:

```bash
cargo lab docs generate
cargo lab check
```

The gate writes the normal ignored `site/` build output. It does not rewrite YAML, generated Markdown or MkDocs navigation. The CLI defaults to the repository where it was compiled; `cargo lab check --root /path/to/checkout` selects another checkout. Every stage executes from the selected root. History uses the existing CI event baseline and `VALIDATION_ARTIFACT_MAP` supplies evidence mirrors. The gate has no repair, skip-tests, capture, registration or deployment option.

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

The separate manually dispatched capture workflow invokes `chronoshft-lab execution capture` with locked offline Cargo and retains observations, including normal failures, without pushing records or approving validations. Shared CI refuses performance-class acceptance cases. Performance acceptance still requires its declared controlled environment, workload and retained observations. Reviewer decisions and architecture approvals remain explicit under the [framework contract](../00-governance/validation-framework.md). Stage 9 supplies the bound EXP-001 capture plan; its sequential observations require separate review. Stage 10 investigates EXP-002 overload behavior.

## Regression migration coverage

`tests/compatibility/rust-coverage.json` maps every one of the 110 frozen Python
regressions to native Rust test assertions. A native inventory check reads the
original test names from the pinned reference Git commit and rejects missing,
duplicate or stale counterpart names. Grouped Rust tests may cover several Python
cases; a mapping or a larger test count alone does not establish behavioral parity.
The native regressions must pass. The Python comparison passed before retirement; its original source and expectations remain pinned in Git.

Native gate tests exercise every failure boundary, launch failures, correct working
directories, strict MkDocs ordering and freshness checks without writing. Workflow
contracts verify relevant triggers, locked dependencies, full history, explicit
committed-plan capture, failure artifact retention and Pages publication conditions.
New tests also fill the remaining registry, navigation and compatibility-view cases.

The primary gate contains no repository-owned Python script or Python unittest
invocation. Python remains necessary for MkDocs. Stage 8 removes custom Python
validation scripts, their regression modules and validation-only dependencies. The
capture workflow needs only Rust; the documentation workflow retains Python/uv.
Current bindings use the root Cargo workspace and lockfile. Frozen generated
fixtures retain their original hashes; an explicit presentation comparison allows
only the new generator attribution and three reproduction commands. Historical
RUN/EVD/VDEC records, artifacts and source commits remain unchanged.
