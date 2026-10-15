# chronoshft-lab

Rust tooling for the Technical Validation Lab migration. Stages 2–8 provide native
registry, Git-history and retained-evidence verification, Markdown/navigation
generation, rendered-link checks, execution capture/import, experiment helpers,
reviewer workflows and the shared `cargo lab check` gate, including strict MkDocs.
Custom Python tools and regression modules are retired. Native plan bindings use
the shared root Cargo workspace/lockfile; MkDocs retains Python/uv.

From the repository root:

```sh
cargo fetch --locked
cargo run --offline --locked -p chronoshft-lab -- registry check --require-tests
cargo lab registry check --help
cargo lab history check
cargo lab evidence verify
cargo lab docs generate --check
cargo lab docs generate
# After the strict MkDocs build:
cargo lab docs links
cargo lab experiment run-case fifo_exactly_once
cargo lab toolchain show
cargo lab execution capture --help
cargo lab execution import --help
cargo lab review prepare --help
cargo lab review check --help
cargo lab check
cargo lab migration status
cargo test --offline --locked --workspace
```

Registry checks return 0 for consistency, 1 for invalid data/read failures and
2 for invalid arguments. All proposed command groups are implemented; the shared gate propagates child
failures and stops before subsequent work. Registry
consistency alone does not verify retained content; use `evidence verify` for
artifact/source/decision checks. Documentation generation changes only generated Markdown and the bounded Lab
navigation; `--check` is read-only. Capture/import publish verified observations
exclusively and never replace existing records. No command grants technical
approval automatically or changes definition/lifecycle records.
`review prepare` writes an editable unapproved package outside the decision
registry; `review check` is read-only unless `--register-reviewed` explicitly
attests a supplied named human review. Registration appends only a new VDEC,
leaving lifecycle and current-decision selection for explicit record edits. Historical verification reads old source and schemas from Git.

The [migration contract](../../docs/10-validation-lab/rust-migration.md) explains
YAML/schema compatibility, frozen fixtures, module mapping and stage boundaries.
MkDocs and its Python documentation dependencies remain supported. The experiment
and tooling now share the root workspace and lockfile.
Native capture supports Unix process groups, retained failure/aborted observations
and identical publication recovery; see the contract for platform limits.

The primary gate runs native checks, offline locked Cargo tests, locked MkDocs and
rendered links from the selected root. Rust/Cargo and `uv` must be on PATH. It
never repairs generated files, captures a project experiment, registers approval
or deploys the site. Native CI tests enforce publication and capture boundaries.
`tests/compatibility/rust-coverage.json` traces the complete frozen Python test
inventory to native assertions; original expectations remain unchanged.
