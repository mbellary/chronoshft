# Frozen Python v1 compatibility reference

These are migration test fixtures, not new validation or approval records.
See [the migration contract](../../../docs/10-validation-lab/rust-migration.md).

`manifest.json` pins the reference commit and records source-input digests,
reference registry/history/evidence outcomes, YAML overlay expectations,
aggregate-outcome scenarios and generated-file hashes. `cases/registry/` contains
10 overlays verified against the Python reference. `golden/` contains 39
Markdown outputs and `mkdocs.yml`, generated from the same pinned input tree.

Restore the pinned tree in a disposable directory/check out its Git commit for
history-sensitive cases. Apply each overlay to its manifest `target` in a fresh
copy, then compare the Rust and Python implementations against the recorded
acceptance and diagnostic category. Do not validate an overlay as an independent
complete registry. Compare generated UTF-8 bytes from the pinned records and
navigation, not the current live registers after they change.

The Python test modules at the pinned commit remain the reference for wider
runtime/process/publication/review behavior. Stages 2–6 add executable Rust parity
checks. Stage 1 does not yet implement those validators or claim Rust parity.

Keep historical fixture versions; an intentional expectation change needs review
and a new/versioned expectation with its reason. Never regenerate this baseline
from the latest checkout just to hide a mismatch.
