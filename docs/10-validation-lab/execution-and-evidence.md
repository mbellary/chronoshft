# Execution, evidence and review workflow

Stage 8 provides tools for pinned observations, retained artifacts, historical integrity and explicit review. It introduces no actual ChronosHFT execution or approval. The initial experiment methods remain unbound; regression fixtures exercise the infrastructure in temporary Git repositories.

## Source and readiness boundary

A real execution requires committed experiment/test definitions, a ready experiment and bound mandatory methods. The capture runner checks that selected automated commands match their pinned bindings and that implementation paths exist at the declared implementation commit. Tested source, inputs and lockfiles must match a clean checkout. Append-only ledger and content-addressed artifact additions are excluded from source dirtiness so several runs can share the same implementation commit.

Registry and implementation commits may differ. `HEAD` is permitted in a capture plan and resolves to an exact commit before execution; RUN/EVD/VDEC records always retain exact hashes. Historical definitions resolve from the pinned Git tree or its retained revision snapshots, using the schemas from that same tree. Uncommitted framework or binding work cannot be reported as a committed experiment result.

Readiness still needs the declared contracts, fixtures, independent oracle and applicable environment. An entrypoint string does not prove those prerequisites are fulfilled. Keep production design approvals distinct from an isolated investigation's supported scope.

## Declare a capture plan

`validation/schemas/capture-plan.schema.json` defines the pre-execution input. A plan records:

- Unique RUN identity; experiment ID/revision and registry commit; implementation commit and repository-relative working directory.
- A toolchain version command, declared configuration/deviations, explicit process variables, input files, lockfiles, seeds and instrumentation.
- Per-command timeout and artifact retention policy with a per-artifact repository size limit.
- Selected test IDs/revisions, exact argument arrays, assertion-failure exit codes and required output paths/kinds.

Commands run as argument arrays with no shell. Selected cases execute in the pinned experiment's order; the recorded procedure lists attempted commands in that actual order, while skipped cases remain explicit outcomes. The runner uses a small platform environment plus declared variables, and records the resolved environment. Keep credentials out of plan/configuration metadata. Toolchain, OS and CPU observations are captured from the actual host; topology and workload settings still need their declared configuration and deviations.

Use `repository` retention for small content-addressed artifacts or `bundle` for separately retained outputs. Set the explicit repository size limit for the intended storage policy. Artifacts above that limit remain in the run bundle instead of entering Git. No ready execution plan is supplied in Stage 8; the first plan follows the first implemented/bound experiment.

After preparing and committing a real plan and its bindings:

```bash
uv run --locked --extra validation python scripts/capture_execution.py \
  --plan validation/execution-plans/EXP-001.yaml \
  --committed-plan --run-id RUN-local-unique
```

The path and ID above are illustrative, not an existing plan or registered run. Choose a fresh ID for every observation. An already reserved bundle or registered identity cannot be reused.

## Capture and preserve observations

The runner writes `evidence/<RUN-ID>/` with the resolved plan, toolchain output, test stdout/stderr, declared outputs, structured results and staged manifests. Large copies and digest checks stream file content. Small repository-retained artifacts live under `validation/artifacts/sha256/<digest>`; existing content is reused only if its digest and size still match.

Every declared experiment test gets an outcome. Unselected tests are explicitly skipped. Exit zero records a process pass, declared assertion exits record failure, and unexpected exits, timeouts or missing required artifacts record errors. Tracked-source/input changes stop further testing and prevent positive acceptance. An interrupt handled during a test records an aborted execution. Missing cases, skips, global tool errors and aborted runs cannot become an aggregate pass.

These are process observations, not proof that a test's oracle or domain coverage is adequate. Independent test logic and required domain evidence remain review obligations. Harnesses must wait for their work and finalize outputs before returning. Forced termination or runner loss can prevent finalization; retain the reserved partial bundle and report only known observations rather than inventing a result.

Publication validates schemas, pinned definitions, per-case/run evidence, inputs, lockfiles and hashes before writing. EVD manifests and the final RUN use exclusive atomic writes and cannot replace existing records. A failed/incomplete/aborted capture is retained and returns a nonzero command status. The mutable `capture-state.json` is an operational journal, not an immutable evidence record.

## Manual observations and interrupted publication

Documentary investigations use bound review checklists and versioned sources. Prepare a complete RUN manifest with manual outcomes, no fabricated executable command/exit code, and retained review artifacts with EVD manifests. Each declared test still needs an outcome, including skips. Import it through the same verification path:

```bash
uv run --locked --extra validation python scripts/register_execution.py \
  --run /retained/manifests/RUN-unique.yaml \
  --evidence-dir /retained/manifests/evidence
```

If publication was interrupted after staging complete verified manifests, use `--resume` with those exact files. Existing records must be identical; recovery writes only missing records. Changed records require new identities and supersession references. A partial bundle without final manifests needs an honest reconstruction/review of known observations; `--resume` does not rerun commands or manufacture outcomes.

## Verify retained content

```bash
uv run --locked --extra validation python scripts/verify_evidence.py
```

Verification checks actual artifact size/SHA-256, pinned experiment/test/validation definitions, test ownership and bindings, complete case accounting, aggregate outcomes, source/input/lockfile hashes and decision evidence. `repo://` references resolve within the repository or an explicit mirror root. External artifact URIs need a local mirror mapping; verification never silently skips inaccessible content or fetches arbitrary URLs.

Use `--artifact-root /retained/root` for a mirror preserving repository-relative paths, or `--artifact-map /retained/map.yaml` for EVD-ID-to-file mappings. A mapping changes the local lookup location, not the registered URI/hash. The documentation gate also accepts `VALIDATION_ARTIFACT_MAP`. Missing or modified evidence fails verification. Retain the logical bundle structure when downloading CI artifacts.

## CI capture and retention

The manually dispatched **Validation experiment capture** workflow takes a committed plan under `validation/execution-plans/`, assigns a unique CI RUN identity and captures bound automated observations. It has read-only repository permissions and does not push records, select decisions or deploy documentation. Shared CI rejects performance-class acceptance cases and records its environment limitation; controlled performance execution remains a separate activity.

The workflow uploads `evidence/` and repository content-addressed artifacts even after a normal capture failure. Downloads preserve the bundle and staged manifests for inspection/import. Its configured 90-day retention is temporary, not permanent validation storage. Move required artifacts to approved retained storage or an accessible mirror before expiry. Publication/acceptance checks fail when referenced content is unavailable. Runner loss may interrupt both capture and upload.

The Documentation workflow fetches history and now includes Git-baseline and evidence/decision verification in its [shared gate](automation-and-ci.md). Repository-retained artifacts are available in a fresh checkout; bundle/external artifacts need an explicit retained mirror. No capture is automatically launched on ordinary pull requests.

## History and revision enforcement

`scripts/check_validation_history.py` compares against the pull-request base SHA, the push's previous SHA, the prior commit on manual dispatch, or `HEAD` for local working changes. An initial event has no earlier baseline; current schema/coverage checks still apply. Unavailable history fails instead of silently using a weaker baseline.

Previously committed RUN/EVD/VDEC records, revision snapshots, content-addressed artifacts, baseline manifests and the supplied assessment are immutable. Stable definition IDs cannot disappear; provenance, revision histories and lifecycle transitions cannot be rewritten. A changed YAML definition requires a revision and a matching retained prior definition. CI treats wording changes in definition fields conservatively as revisions; YAML formatting and authored-document presentation changes do not alter the parsed definition.

These checks enforce append-only behavior relative to the trusted Git baseline. File permissions and hashes alone are not an authorization system; repository review and access controls remain responsible for who may change tooling or approve records.

## Prepare and register a human decision

```bash
uv run --locked --extra validation python scripts/review_validation.py prepare \
  --validation VAL-012 --decision-id VDEC-001 --output /tmp/VAL-012-review.yaml
```

This produces an unapproved draft outside the registry. Reviewer, disposition, rationale and scope remain unassigned. Available runs/evidence are an inventory, not an automatic selection. A human must select applicable observations, exact registry/implementation boundaries and evidence; fill the requirement-by-requirement assessment; record unresolved findings, operating limits and follow-up; and attest their identity and review time. Set `draft: false` only after that review.

```bash
uv run --locked --extra validation python scripts/review_validation.py check \
  --candidate /tmp/VAL-012-review.yaml
```

The default check writes no decision. Accepted or conditional conclusions require clean, valid observations covering every mandatory applicable test and experiment, every evidence obligation mapped to verified artifacts, a concrete operating envelope and no invalidating finding. Later contrary observations within the same implementation/definition/input/environment boundary cannot be hidden by selecting an earlier pass. Conditional acceptance additionally needs explicit limits and follow-up; it cannot waive tests without revising the declared scope. Rejected/inconclusive decisions retain evidence and follow-up without requiring a passing hypothesis.

After the named human review, the operator may explicitly add `--register-reviewed` to append the decision. The validation must already be in `reviewing`. The tool validates supplied attestation fields but does not authenticate a person's identity or grant approval itself. No real reviewer is assigned by this implementation.

Registration writes only VDEC. Current-decision selection, prior-decision retention and legal lifecycle closure/reopening are explicit subsequent record edits. Architecture adoption remains in the [decision register](../00-governance/decision-register.md). Historical decisions retain their original applicability after revalidation; they do not automatically establish current acceptance.

## First executable experiment

Stage 9 binds EXP-001 to a safe mutex-backed Rust prototype. Its provisional
contract uses individual events, supports exactly N usable slots for every
integer N in 1..=4096 and returns the original event when full before changing
a slot. It does not resolve production architecture or overload decisions.

From the repository root, run all four correctness tests:

```sh
cargo test --offline --locked --manifest-path experiments/Cargo.toml -p exp-001-spsc-queue
```

After committing source, definitions and the plan, capture one observation set:

```sh
uv run --locked --extra validation python scripts/capture_execution.py \
  --plan validation/execution-plans/EXP-001.yaml --committed-plan \
  --run-id RUN-exp001-local-001
```

Use a new RUN identity for every subsequent capture. The plan records actual
rustc/cargo versions, committed model schedules and lockfile, seed 24301,
per-operation ledgers, constructor outcomes and stdout/stderr. The bound runner
builds first and checks that the exact case exists: compile/launch errors return
2; Rust assertion failures return 101. Missing tests cannot produce an empty pass.
Reports are recreated for each invocation to avoid retaining stale observations.

Read the generated [EXP-001 page](experiments/EXP-001.md) and
[execution register](execution-register.md) for registered results and source
pins. Passing these cases covers sequential behavior only. Mutex contention,
concurrent publication, allocator failure injection, disconnection and performance
remain outside this experiment. VAL-012 and VAL-013 remain pending.

## Next stage

Stage 10 implements EXP-002, the full-ring and slow-consumer overload investigation,
with an explicit provisional policy, accounting tests and retained observations.
EXP-003 still owns concurrent publication and ownership proof.
