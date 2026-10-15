//! Isolated correctness prototype. Mutex synchronization is intentional;
//! these sequential experiments establish no lock-free or latency guarantee.
#![forbid(unsafe_code)]
use std::sync::{Arc, Mutex};

pub const MAX_CAPACITY: usize = 4096;

#[derive(Debug, PartialEq, Eq)]
pub enum CapacityError {
    Zero,
    TooLarge,
    AllocationFailed,
}

struct Ring<T> {
    slots: Vec<Option<T>>,
    read: usize,
    write: usize,
    occupied: usize,
}

/// One producer endpoint; deliberately neither Clone nor Sync.
pub struct Producer<T> {
    ring: Arc<Mutex<Ring<T>>>,
    single_owner: std::marker::PhantomData<std::cell::Cell<()>>,
}

/// One consumer endpoint; deliberately neither Clone nor Sync.
pub struct Consumer<T> {
    ring: Arc<Mutex<Ring<T>>>,
    single_owner: std::marker::PhantomData<std::cell::Cell<()>>,
}

/// N counts individual events. All N slots are usable, including at N=1.
/// Every integer capacity in 1..=MAX_CAPACITY is supported.
/// Invalid sizes are rejected before reserving storage.
pub fn bounded<T>(capacity: usize) -> Result<(Producer<T>, Consumer<T>), CapacityError> {
    if capacity == 0 {
        return Err(CapacityError::Zero);
    }
    if capacity > MAX_CAPACITY {
        return Err(CapacityError::TooLarge);
    }
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(capacity)
        .map_err(|_| CapacityError::AllocationFailed)?;
    slots.resize_with(capacity, || None);
    let ring = Arc::new(Mutex::new(Ring {
        slots,
        read: 0,
        write: 0,
        occupied: 0,
    }));
    Ok((
        Producer {
            ring: Arc::clone(&ring),
            single_owner: std::marker::PhantomData,
        },
        Consumer {
            ring,
            single_owner: std::marker::PhantomData,
        },
    ))
}

impl<T> Producer<T> {
    /// Returns the original event on full, before modifying any slot.
    /// Both endpoints require exclusive mutable access for operations.
    pub fn try_push(&mut self, event: T) -> Result<(), T> {
        let mut ring = self.ring.lock().expect("queue mutex poisoned");
        if ring.occupied == ring.slots.len() {
            return Err(event);
        }
        let position = ring.write;
        ring.slots[position] = Some(event);
        ring.write = if position + 1 == ring.slots.len() {
            0
        } else {
            position + 1
        };
        ring.occupied += 1;
        Ok(())
    }

    pub fn capacity(&self) -> usize {
        self.ring.lock().expect("queue mutex poisoned").slots.len()
    }
}

impl<T> Consumer<T> {
    pub fn try_pop(&mut self) -> Option<T> {
        let mut ring = self.ring.lock().expect("queue mutex poisoned");
        if ring.occupied == 0 {
            return None;
        }
        let position = ring.read;
        let event = ring.slots[position].take();
        ring.read = if position + 1 == ring.slots.len() {
            0
        } else {
            position + 1
        };
        ring.occupied -= 1;
        event
    }
}
