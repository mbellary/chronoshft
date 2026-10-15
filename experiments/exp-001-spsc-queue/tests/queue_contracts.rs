use exp_001_spsc_queue::{bounded, CapacityError, Consumer, Producer, MAX_CAPACITY};
use std::collections::VecDeque;
use std::fs;

// Reports are created before assertions and appended after each observation so
// assertion failures retain the observed prefix. Logs are independently captured.
struct Report {
    path: std::path::PathBuf,
}
impl Report {
    fn new(id: &str) -> Self {
        let directory = std::env::var_os("CHRONOSHFT_REPORT_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("target/validation-reports"));
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(directory);
        fs::create_dir_all(&directory).unwrap();
        let path = directory.join(format!("{id}.txt"));
        fs::write(
            &path,
            format!("{id}: sequential bounded queue observations\n"),
        )
        .unwrap();
        Self { path }
    }
    fn record(&self, value: impl AsRef<str>) {
        use std::io::Write;
        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&self.path)
            .unwrap();
        writeln!(file, "{}", value.as_ref()).unwrap();
    }
}

struct Model {
    producer: Producer<u64>,
    consumer: Consumer<u64>,
    capacity: usize,
    expected: VecDeque<u64>,
    accepted: Vec<u64>,
    consumed: Vec<u64>,
    report: Report,
}
impl Model {
    fn new(capacity: usize, report: Report) -> Self {
        let (producer, consumer) = bounded(capacity).unwrap();
        assert_eq!(producer.capacity(), capacity);
        report.record(format!("capacity={capacity}"));
        Self {
            producer,
            consumer,
            capacity,
            expected: VecDeque::new(),
            accepted: vec![],
            consumed: vec![],
            report,
        }
    }
    fn push(&mut self, id: u64) {
        let expect_full = self.expected.len() == self.capacity;
        let actual = self.producer.try_push(id);
        self.report
            .record(format!("push {id}: {actual:?}; model_full={expect_full}"));
        if expect_full {
            assert_eq!(actual, Err(id));
        } else {
            assert_eq!(actual, Ok(()));
            self.expected.push_back(id);
            self.accepted.push(id);
        }
    }
    fn pop(&mut self) {
        let expected = self.expected.pop_front();
        let actual = self.consumer.try_pop();
        self.report
            .record(format!("pop: {actual:?}; model={expected:?}"));
        assert_eq!(actual, expected);
        if let Some(id) = actual {
            self.consumed.push(id);
        }
    }
    fn finish(&mut self) {
        while !self.expected.is_empty() {
            self.pop();
        }
        self.pop();
        self.pop();
        self.report.record(format!(
            "accepted={:?}\nconsumed={:?}",
            self.accepted, self.consumed
        ));
        assert_eq!(self.consumed, self.accepted);
        self.report
            .record("PASS: exact FIFO, exactly once, drained empty");
    }
}

#[test]
fn fifo_exactly_once() {
    let mut m = Model::new(7, Report::new("TST-001"));
    m.pop();
    for round in 0..64 {
        m.push(round * 3);
        m.push(round * 3 + 1);
        m.pop();
        m.pop();
        m.pop();
        m.push(round * 3 + 2);
        m.pop();
    }
    m.finish();
}

#[test]
fn capacity_before_write() {
    let report = Report::new("TST-002");
    for capacity in [1, 2, 3, 7, 32] {
        let mut m = Model::new(
            capacity,
            Report {
                path: report.path.clone(),
            },
        );
        for id in 0..capacity as u64 {
            m.push(id);
        }
        for id in 10_000..10_005 {
            m.push(id);
        }
        m.pop();
        m.push(20_000);
        m.push(20_001);
        m.finish();
        for id in 30_000..30_000 + capacity as u64 {
            m.push(id);
        }
        m.push(40_000);
        m.finish();
    }
}

#[test]
fn wraparound_partial_drains() {
    let report = Report::new("TST-003");
    let initial_seed = 0x5eed_u64;
    report.record(format!("seed={initial_seed}; index_width={}; bounded slot indexes; no monotonic counter rollover claim", usize::BITS));
    for capacity in [1, 3, 4, 7, 32] {
        let mut m = Model::new(
            capacity,
            Report {
                path: report.path.clone(),
            },
        );
        let mut seed = initial_seed;
        // A deterministic prefix guarantees full attempts, partial drains and
        // at least eight complete physical traversals for each capacity.
        let mut id = 0;
        for _ in 0..16 {
            for _ in 0..capacity + 2 {
                m.push(id);
                id += 1;
            }
            for _ in 0..capacity.div_ceil(2) {
                m.pop();
            }
        }
        for _ in 0..512 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            if seed >> 63 == 0 {
                m.push(id);
                id += 1;
            } else {
                m.pop();
            }
        }
        m.finish();
        let traversals = m.accepted.len() / capacity;
        m.report.record(format!(
            "complete_write_traversals={traversals}; accepted={}",
            m.accepted.len()
        ));
        assert!(traversals >= 4);
    }
}

#[test]
fn capacity_configuration() {
    let report = Report::new("TST-004");
    // Checked-in experiment contract, independent of implementation helpers.
    let contract_max = 4096_usize;
    assert_eq!(MAX_CAPACITY, contract_max);
    for capacity in [0, 1, 2, 3, 4096, 4097, usize::MAX] {
        let expected = if capacity == 0 {
            Err(CapacityError::Zero)
        } else if capacity > contract_max {
            Err(CapacityError::TooLarge)
        } else {
            Ok(())
        };
        let actual = bounded::<u64>(capacity);
        report.record(format!(
            "requested={capacity}; expected={expected:?}; actual={:?}",
            actual.as_ref().map(|(p, _)| p.capacity())
        ));
        match expected {
            Err(error) => assert!(matches!(actual, Err(found) if found == error)),
            Ok(()) => {
                let (mut producer, mut consumer) = actual.unwrap();
                assert_eq!(producer.capacity(), capacity);
                let mut accepted = vec![];
                for id in 0..capacity as u64 {
                    assert_eq!(producer.try_push(id), Ok(()));
                    accepted.push(id);
                }
                assert_eq!(producer.try_push(u64::MAX), Err(u64::MAX));
                let consumed: Vec<_> = (0..capacity).map(|_| consumer.try_pop().unwrap()).collect();
                assert_eq!(consumed, accepted);
                assert_eq!(consumer.try_pop(), None);
                report.record(format!("exact_usable={capacity}; extra_returned; FIFO drain verified; bytes_checked={:?}", capacity.checked_mul(std::mem::size_of::<Option<u64>>())));
            }
        }
    }
    report.record("PASS: zero and oversized rejected; all integer sizes including non-powers-of-two supported within 1..=4096; allocation failure injection excluded");
}
