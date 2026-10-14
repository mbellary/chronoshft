# Security

!!! info "Research-derived operations draft · 2026-10-14"
    Based on the [HFT research notes](../assets/research/HFT-Research-Notes.pdf) (BASE-001, physical PDF pages) and the Product, Architecture and Engineering contracts linked below. Source proposals and derived operational procedures remain distinct. No implemented control, approved SLA, recovery guarantee or measured performance is asserted.


## Source boundary

Private VPC/co-located operation without runtime external Internet dependencies is the PDF's security requirement (p. 17; RES-015; PRD-N04). It does not define authentication, authorization, encryption, secret management or a compliance certification. PRD-N05 and ENG-015 are derived candidates that require concrete controls and tests; GAP-015/018 remain open.

## Proposed trust boundaries and controls

| Boundary | Exposure | Control to specify and verify |
| --- | --- | --- |
| Feed → adapter | Untrusted framing/messages, oversized inputs, malformed values and entitlement/session state | Allowed venue/version and peers, bounded framing/decoding, credential handling, rejection accounting and gap/replay policy |
| Client → SQL/Flight | Unauthorized data access, expensive queries and slow streams | Authentication, dataset/query permissions, transport protection, query/result/concurrency limits, cancellation and partial-error behavior |
| Administrator → configuration | Changes to feeds, access, retention, durability and resource limits | Separate administrative permissions, configuration validation, auditable changes and restricted secret access |
| Process → data/catalog/logs | Local disclosure, deletion/corruption and credential leakage | Least-privilege filesystem/service access, selected at-rest protection, log redaction and backup access restrictions |
| Package/dependencies → runtime | Incompatible or untrusted artifacts; hidden network dependencies | Pinned compatible versions, verified package inventory/checksums and offline build/start evidence |

These are proposed control categories, not implemented mechanisms or selected protocols. The research's kernel-bypass direction does not settle TLS/framing for every adapter (VAL-025; DEC-011), and Flight transport does not establish a complete authorization policy (DEC-013).

## Proposed operational practices

Keep feed, query and administrative access distinct. Document who can view instruments/history, request costly workloads, change loss/retention settings and read raw Parquet or backups. Raw-file permissions can bypass query-level controls and correction-aware views.

Select secure credential provisioning, rotation/revocation and failure behavior; do not embed secrets in configuration examples, metrics or diagnostics. Restrict endpoints to approved networks/peers and exercise unauthorized/expired credential cases. Select transport and at-rest protections against the actual trust model rather than relying on VPC placement.

Offline acceptance uses local fixtures and approved packages with external services unavailable; it does not prove a public venue feed can operate disconnected. Optional object storage, remote management and external identity/telemetry services each need an explicit connectivity and trust review before inclusion.

## Abuse, data integrity and evidence

Bound parser input, memory, query execution and result retention. Test malformed/oversized messages, unauthorized access, invalid configuration, client disconnect and resource exhaustion against declared failure policies. Keep gap/correction status visible so incomplete data cannot silently appear complete (PRD-N06/N08).

Record build/config identity, authorization failures and administrative changes under a bounded audit/retention policy. Market-data licensing, redistribution entitlements and retention obligations need owner review; no jurisdictional or certification claim is established here.

## Review dependencies

[Product requirements](../01-product/product-requirements.md) PRD-N04/N05; [platform design](../02-architecture/platform-design.md); [Flight/Python contract](../03-engineering/flight-and-python.md); [ingestion contract](../03-engineering/ingestion-engine.md); ENG-002/006/013/015. Security ownership, concrete authentication/authorization, encryption, secret lifecycle and audit acceptance remain unresolved. Track evidence through [operational readiness](readiness-and-traceability.md), GAP-015/018 and DEC-002/013.
