# Recovery

!!! info "Research-derived operations draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages) and the Product, Architecture and Engineering contracts linked below. Source proposals and derived operational procedures remain distinct. No implemented control, approved SLA, recovery guarantee or measured performance is asserted.


## Durability and failure boundary

The PDF proposes background micro-Parquet flushing (pp. 9, 16, 27) and late/corrected history (pp. 17, 28). It supplies no complete crash-consistent commit protocol, durable ingest acknowledgment, RPO/RTO or backup/restore contract (RES-018/030; VAL-024; DEC-008). [Persistence engineering](../03-engineering/persistence-engine.md) defines candidate modes; none is selected.

| Candidate mode | Recovery boundary to declare |
| --- | --- |
| Volatile admission / asynchronous flush | Accepted or published but uncommitted events may be lost; identify recoverable committed history |
| WAL-backed admission | Define log commit/ack ordering, replay identity, duplicate prevention and safe truncation |
| Source-replay recovery | Verify source retention/session replay availability and reconcile admitted/visible/committed ranges |

Process crash, power loss, corruption and total disk loss require separate evidence. Two striped NVMe disks provide no redundancy; active/passive HA is a later source proposal (pp. 17–18), not an existing automatic recovery path.

## Proposed incident procedures

| Failure | Containment and recovery proposal | Required verification |
| --- | --- | --- |
| Feed gap/disconnect | Record venue/session and missing range; apply approved replay/resync/quarantine or explicitly degraded mode | Sequence continuity, duplicates/corrections and affected query coverage; ENG-002/007 |
| Full ring or slow consumer | Apply declared capacity policy before append; identify query/stream/flusher retention | Every loss/rejection accounted; no overwrite of live data; ENG-003–006 |
| Disk full/permission/write failure | Keep unfinished output out of catalog; retain bounded ownership and apply selected intake policy | Failed writes never masquerade as committed; backlog remains bounded; ENG-010 |
| Crash during file/catalog commit | Reconcile partial/orphan files and committed metadata using the selected protocol | Promised committed events survive exactly once; ENG-010/011 |
| Corruption or disk loss | Preserve evidence, restrict affected history and use validated restore or available source replay | Declared coverage/version checks; disclose unrecoverable ranges |
| Failed compaction/correction update | Recover the selected atomic catalog generation; defer deletion while snapshots exist | Current/as-known oracle remains correct; ENG-011/012 |
| Flight/client failure | Cancel execution, release references and expose incomplete-stream error | Partial data cannot appear successfully complete; ENG-013 |

These procedures depend on ratified policies; no restart command, WAL format or automatic repair feature is implied.

## Proposed restart sequence

1. Preserve incident/config/build identity and failure evidence; establish the selected acknowledgment/failure model and affected ranges.
2. Inspect storage and committed catalog under that model. Classify incomplete/orphan outputs; do not blindly promote or delete them.
3. Restore eligible history/catalog or replay the supported source/log, preserving event identity, ordering and correction knowledge versions.
4. Reconcile hot/cold coverage so each eligible logical event/version is selected once; never deduplicate solely by timestamp.
5. Run reference SQL/type/correction checks and establish the recovered visibility boundary before opening queries or intake.
6. Resume under bounded resource policies and disclose any lost or degraded ranges. Record recovered coverage and elapsed recovery time.

A historical as-known view depends on preserved knowledge-time metadata; replay time must not silently replace original knowledge time (DEC-009). New live sessions need explicit sequence/session handling.

## Backup, restore and retention proposal

Select backup cadence, location, encryption/access and retention after the failure model is chosen. A restorable set must contain mutually consistent committed files, catalog/version/schema metadata and required configuration; select WAL/checkpoint coverage if that mode is adopted. Copying files during compaction alone does not prove consistency.

Rehearse restoration into an isolated environment, verify coverage/counts/types and current/as-known results, and measure recovery against approved objectives. Keep backup retention and deletion coordinated with active snapshots, corrections and entitlement requirements. No numeric RPO/RTO or zero-loss promise is assigned by these docs.

## Recovery acceptance

Inject failures before file completion, before/after catalog publication, during retry, hot eviction and compaction; test disk-full, corruption, permission failure and restart. Verify accepted/published/durable boundaries, ownership release, gap accounting and financial correctness. PRD-N06–N09; ENG-002/005/010–013; DEC-003/006/008/009; VAL-023/024. Related: [query/storage](../02-architecture/query-and-storage.md), [Engineering validation](../03-engineering/validation-and-traceability.md), [observability](observability.md) and [readiness](readiness-and-traceability.md).
