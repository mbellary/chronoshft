---
hide:
  - toc
---

# ChronosHFT

<div class="chronos-hero" markdown>

<span class="chronos-eyebrow">RESEARCH · ARCHITECTURE · ENGINEERING</span>

## Financial data. Explicit contracts. Measured performance.

The documentation workspace for a proposed Rust financial time-series database and analytics engine, built around Apache Arrow Flight, DataFusion, Arrow and Parquet.

[Start with the research](00-governance/research-baseline.md){ .md-button .md-button--primary }
[Explore the architecture](02-architecture/system-architecture.md){ .md-button }

</div>

!!! info "Documentation foundation"
    Research & governance, Product, Architecture, Engineering, Rust reference, Benchmarking & validation and Operations contain substantive drafts and learning references grounded in the supplied research and companion documents. Roadmap contains a proposed implementation plan, MVP delivery gates and milestone registry. Financial domain and Architecture decisions remain draft outlines. Design proposals, operational procedures and performance targets require review and evidence; they are not approved specifications or verified results.

## Explore the documentation

<div class="grid cards" markdown>

-   **A · Research & governance**

    ---

    Explore the Phase 0 sources, extraction, technical validation, performance profiles, open gaps, decisions and readiness gates.

    [Research baseline](00-governance/research-baseline.md) · [Research assessment](00-governance/research-assessment.md) · [Research extraction matrix](00-governance/research-extraction.md) · [Technical validation](00-governance/technical-validation.md) · [Performance profiles](00-governance/performance-profiles.md) · [Assumptions & gaps](00-governance/assumptions-and-gaps.md) · [Decision register](00-governance/decision-register.md) · [Traceability](00-governance/traceability.md) · [Readiness & handoff](00-governance/readiness-and-handoff.md)

-   **B · Product**

    ---

    Explore market context, requirements, product architecture, MVP scope, benchmark targets and the Phase 2 handoff.

    [Phase 2 overview](01-product/phase-2-definition.md) · [Market landscape & FDAP](01-product/market-landscape-and-fdap.md) · [Product requirements](01-product/product-requirements.md) · [Product architecture](01-product/product-architecture.md) · [Scope & MVP](01-product/scope-and-mvp.md) · [Benchmark targets](01-product/benchmark-targets.md) · [Source-to-product traceability](01-product/product-traceability.md) · [Open-decision handoff](01-product/product-decision-handoff.md) · [Exit assessment & readiness](01-product/product-readiness.md)

-   **C · Architecture**

    ---

    Follow the system and platform design, component contracts, data flow, active memory, query integration and deployment topology.

    [System architecture](02-architecture/system-architecture.md) · [Platform design](02-architecture/platform-design.md) · [Component contracts](02-architecture/component-contracts.md) · [Data flow](02-architecture/data-flow.md) · [Active memory & concurrency](02-architecture/active-memory.md) · [Query & storage integration](02-architecture/query-and-storage.md) · [Deployment topology](02-architecture/deployment-topology.md)

-   **D · Engineering**

    ---

    Examine engineering requirements, subsystem designs, event schemas, financial semantics, client integration and validation traceability.

    [Engineering requirements](03-engineering/engineering-requirements.md) · [Engineering design](03-engineering/engineering-design.md) · [Ingestion engine](03-engineering/ingestion-engine.md) · [Active memory lifecycle](03-engineering/active-memory-lifecycle.md) · [Query engine](03-engineering/query-engine.md) · [Persistence engine](03-engineering/persistence-engine.md) · [Event schema & sequencing](03-engineering/event-schema-and-sequencing.md) · [Financial query semantics](03-engineering/financial-query-semantics.md) · [Flight & Python integration](03-engineering/flight-and-python.md) · [Validation & traceability](03-engineering/validation-and-traceability.md)

-   **E · Rust reference**

    ---

    Connect Rust fundamentals, ownership, error handling, engine patterns, atomics, concurrency, memory management and FDAP integration to practical exercises.

    [Fundamentals](04-rust/fundamentals.md) · [Ownership & lifetimes](04-rust/ownership-and-lifetimes.md) · [Idioms & error handling](04-rust/idioms-and-errors.md) · [Engine design patterns](04-rust/engine-design-patterns.md) · [Atomics & memory ordering](04-rust/atomics-and-memory-ordering.md) · [Lock-free patterns](04-rust/lock-free-patterns.md) · [Concurrency patterns](04-rust/concurrency-patterns.md) · [Memory management](04-rust/memory-management.md) · [FDAP integration](04-rust/fdap-integration.md) · [Reference exercises](04-rust/reference-exercises.md)

-   **F · Financial domain**

    ---

    Understand market data protocols, timestamps, sequencing, order book reconstruction and financial query semantics.

    [Market data protocols](05-financial-domain/market-data-protocols.md) · [Timestamps & sequencing](05-financial-domain/timestamps-and-sequencing.md) · [Order book reconstruction](05-financial-domain/order-book-reconstruction.md) · [Financial query semantics](05-financial-domain/financial-query-semantics.md)

-   **G · Benchmarking & validation**

    ---

    Define reproducible benchmarks, workloads, test strategies, hardware manifests, acceptance criteria and evidence traceability.

    [Benchmark methodology](06-benchmarking/benchmark-methodology.md) · [Workload definitions](06-benchmarking/workload-definitions.md) · [Test strategy](06-benchmarking/test-strategy.md) · [Hardware setup](06-benchmarking/hardware-setup.md) · [Acceptance criteria](06-benchmarking/acceptance-criteria.md) · [Evidence & traceability](06-benchmarking/evidence-and-traceability.md)

-   **H · Operations**

    ---

    Review deployment, observability, security, recovery, capacity planning and operational readiness.

    [Deployment](07-operations/deployment.md) · [Observability](07-operations/observability.md) · [Security](07-operations/security.md) · [Recovery](07-operations/recovery.md) · [Capacity & overload planning](07-operations/capacity-planning.md) · [Readiness & traceability](07-operations/readiness-and-traceability.md)

-   **I · Architecture decisions**

    ---

    Review architecture decision records and the proposed product boundary.

    [ADR-0001: Product boundary](08-decisions/ADR-0001-product-boundary.md)

-   **J · Roadmap**

    ---

    Follow implementation dependencies, delivery milestones and readiness gates.

    [Implementation roadmap](09-roadmap/implementation-roadmap.md) · [MVP delivery plan](09-roadmap/mvp.md) · [Milestone registry](09-roadmap/milestone-register.md)

</div>

## Documentation map

| Section | Responsibility | Entry point |
| --- | --- | --- |
| 00 · Governance | Sources, assessment, decisions and traceability | [Research baseline](00-governance/research-baseline.md) |
| 01 · Product | Requirements, capabilities and MVP boundary | [Product requirements](01-product/product-requirements.md) |
| 02 · Architecture | Runtime topology, interfaces and data flow | [System architecture](02-architecture/system-architecture.md) |
| 03 · Engineering | Algorithms, invariants and subsystem design | [Engineering requirements](03-engineering/engineering-requirements.md) |
| 04 · Rust | Language concepts and implementation patterns | [Rust fundamentals](04-rust/fundamentals.md) |
| 05 · Financial domain | Protocols, sequencing, order books and query semantics | [Market data protocols](05-financial-domain/market-data-protocols.md) |
| 06 · Benchmarking | Workloads, tests, hardware and acceptance gates | [Benchmark methodology](06-benchmarking/benchmark-methodology.md) |
| 07 · Operations | Deployment, observability, security and recovery | [Deployment](07-operations/deployment.md) |
| 08 · Decisions | Architecture proposals and approval records | [ADR-0001: Product boundary](08-decisions/ADR-0001-product-boundary.md) |
| 09 · Roadmap | Dependencies, milestones and implementation readiness | [Implementation roadmap](09-roadmap/implementation-roadmap.md) |

## From research to evidence

```mermaid
flowchart LR
    R[Original research] --> A[Assessment and decisions]
    A --> P[Approved requirements]
    P --> D[Approved designs]
    D --> I[Rust implementation]
    I --> V[Tests and benchmarks]
```

This is the proposed documentation authority hierarchy. Each stage needs explicit evidence and review before it authorizes the next.

## Read and contribute

Begin with the [research assessment](00-governance/research-assessment.md), then follow product definition, architecture, engineering design and validation. Keep source claims, proposed improvements and approved decisions distinct. See the [decision register](00-governance/decision-register.md) and [traceability](00-governance/traceability.md) for the populated Phase 0 records.

The source plan is preserved in the [research directory](https://github.com/mbellary/chronoshft/tree/main/research). The [repository README](https://github.com/mbellary/chronoshft#readme) explains local preview and publication.
