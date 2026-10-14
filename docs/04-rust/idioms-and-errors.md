# Rust idioms and error handling

!!! info "HFT-focused learning reference · 2026-10-14"
    Based on [BASE-001 research](../assets/research/HFT-Research-Notes.pdf), [Product](../01-product/product-requirements.md), [Architecture](../02-architecture/system-architecture.md) and [Engineering](../03-engineering/engineering-requirements.md). PDF references use physical pages. Examples teach proposed engineering invariants; they do not approve open decisions, implement the production engine or demonstrate performance targets.

## Make invalid states difficult to express

A **newtype** wraps an existing type to give it a distinct meaning. `EventNs(i64)` and `ReceiveNs(i64)` prevent accidentally swapping two otherwise identical integers. An instrument ID, source sequence and batch generation deserve distinct types when mixing them would corrupt results. Keep fields private and expose validated constructors where a domain invariant exists.

An **enum** gives each outcome a name and associated data. `Option<Batch>` can mean that an empty timer produced nothing; `Result<Batch, BuildError>` means a requested construction succeeded or failed. Never collapse malformed input, an empty queue and a disconnected worker into the same success-like outcome (ENG-002/004/006).

## A complete bounded-admission example

```rust
#[derive(Debug, PartialEq)]
enum AdmitError { Full, InvalidQuantity }

struct Writer { rows: Vec<u64>, max_rows: usize }

impl Writer {
    fn new(max_rows: usize) -> Result<Self, &'static str> {
        if max_rows == 0 { return Err("capacity must be positive"); }
        Ok(Self { rows: Vec::with_capacity(max_rows), max_rows })
    }

    fn append(&mut self, quantity: u64) -> Result<(), AdmitError> {
        if quantity == 0 { return Err(AdmitError::InvalidQuantity); }
        if self.rows.len() == self.max_rows { return Err(AdmitError::Full); }
        self.rows.push(quantity);
        Ok(())
    }
}

fn main() -> Result<(), &'static str> {
    let mut writer = Writer::new(1)?;
    assert_eq!(writer.append(100), Ok(()));
    assert_eq!(writer.append(200), Err(AdmitError::Full));
    assert_eq!(writer.rows, vec![100]);
    Ok(())
}
```

The `?` operator returns an error to the caller, converting it when the required conversion exists. `&mut self` makes append require exclusive access. The logical capacity check occurs before mutation; reserved vector capacity is not admission policy. Zero quantity rejection is an example policy only. Production also preserves the rejected event or defines its explicit disposition, validates every field and appends complete aligned columns (ENG-003/007; DEC-006/014).

## Error boundaries

| Boundary | Expected error | Proposed response to define |
| --- | --- | --- |
| Feed decoder | Malformed, unsupported, truncated | Count and reject/quarantine under adapter policy |
| Sequence tracker | Gap or duplicate | Resync/replay or explicit degraded state |
| Admission | Capacity exhausted | Preserve pending ownership and apply selected overload policy |
| Batch construction | Unequal column lengths/schema mismatch | Keep batch unpublished and report invariant failure |
| Persistence | Disk full, incomplete write | Retain retry ownership; never publish incomplete catalog entry |
| Query/Flight | Cancellation, execution or stream failure | Release leases and report incomplete completion |

Use typed errors inside subsystems and add context at their boundaries: instrument/session, batch identity and operation. Avoid formatting a fresh diagnostic string for every successful tick. `unwrap()` and `expect()` are suitable for known teaching fixtures; feed input and storage failures need handled outcomes. Panics are not a normal overload strategy, and recovering from a panic does not automatically restore column consistency.

## Idioms that support the data path

Use `TryFrom` for checked narrowing, `checked_add`/`checked_mul` for semantic overflow, and exhaustive `match` for lifecycle outcomes. `if let` is convenient for one interesting enum alternative, but do not silently discard its failure branches when they require accounting.

`iter()` borrows elements, `iter_mut()` borrows them exclusively and `into_iter()` consumes the collection. A filter/map pipeline can describe validation or aggregation; `collect::<Vec<_>>()` builds an allocation. Choose clarity first, then measure the chosen append path. Do not replace safe indexing with unchecked access merely because it looks faster.

`std::mem::take(&mut rows)` transfers the old collection out and leaves its default value; replacement storage may allocate later. This is useful for understanding freeze/transfer ownership, not proof of zero-allocation turnover. Resource guards that release on scope exit implement RAII; long-lived tasks still need explicit cancellation and joining.

See [Rust iteration](https://doc.rust-lang.org/book/ch13-02-iterators.html), [ingestion](../03-engineering/ingestion-engine.md), [persistence](../03-engineering/persistence-engine.md) and [schema](../03-engineering/event-schema-and-sequencing.md). Source basis: pp. 16–17, 29–32; ENG-002–007/010/013; VAL-012/014/024.
