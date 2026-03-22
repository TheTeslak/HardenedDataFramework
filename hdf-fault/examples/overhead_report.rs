use std::cell::RefCell;
use std::hint::black_box;
use std::mem::size_of;
use std::rc::Rc;
use std::time::{Duration, Instant};

use hdf_core::{Hardened, InlineStore, ReplicaStore, Tmr};
use hdf_fault::corrupt_slot;
use hdf_layout::{ComplementedStore, RegionId, ReplicaPlacement, SplitStore};
use hdf_sync::{CriticalSection, CriticalSectionHardened, SerializedHardened};

const READ_ITERATIONS: usize = 500_000;
const CHECK_ITERATIONS: usize = 500_000;
const WRITE_ITERATIONS: usize = 250_000;
const REPAIR_ITERATIONS: usize = 200_000;
const BASE_VALUE: u32 = 0x1234_5678;

#[derive(Clone)]
struct SharedStore<T, const N: usize> {
    replicas: Rc<RefCell<[T; N]>>,
}

impl<T: Copy, const N: usize> SharedStore<T, N> {
    fn new(initial: T) -> Self {
        Self {
            replicas: Rc::new(RefCell::new([initial; N])),
        }
    }
}

impl<T: Copy, const N: usize> ReplicaStore<T, N> for SharedStore<T, N> {
    fn read_slot(&self, index: usize) -> T {
        self.replicas.borrow()[index]
    }

    fn write_slot(&self, index: usize, value: T) {
        self.replicas.borrow_mut()[index] = value;
    }
}

struct NoopCriticalSection;

impl CriticalSection for NoopCriticalSection {
    fn enter<R, F>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        f()
    }
}

struct BenchmarkResult {
    label: &'static str,
    iterations: usize,
    elapsed: Duration,
}

impl BenchmarkResult {
    fn ns_per_op(&self) -> f64 {
        self.elapsed.as_secs_f64() * 1_000_000_000.0 / self.iterations as f64
    }
}

fn measure<R>(
    label: &'static str,
    iterations: usize,
    mut f: impl FnMut(usize) -> R,
) -> BenchmarkResult {
    let start = Instant::now();
    for iteration in 0..iterations {
        black_box(f(iteration));
    }
    BenchmarkResult {
        label,
        iterations,
        elapsed: start.elapsed(),
    }
}

fn print_results(title: &str, results: &[BenchmarkResult]) {
    println!("\n{title}");
    for result in results {
        println!(
            "- {:34} {:>8} iters  {:>10.2} ns/op  total {:?}",
            result.label,
            result.iterations,
            result.ns_per_op(),
            result.elapsed
        );
    }
}

fn placement() -> ReplicaPlacement<3> {
    ReplicaPlacement::new([RegionId(0), RegionId(1), RegionId(2)])
}

fn main() {
    println!("HDF host overhead baseline");
    println!("target: host std environment");
    println!("repair benchmarks inject one corrupted replica before each repair()\n");

    let mut core_inline = Hardened::<u32, Tmr, _, 3>::new(BASE_VALUE, InlineStore::new(0));
    let inline_results = [
        measure("core-inline read_checked()", READ_ITERATIONS, |_| {
            core_inline.read_checked()
        }),
        measure("core-inline check()", CHECK_ITERATIONS, |_| {
            core_inline.check()
        }),
        measure("core-inline write()", WRITE_ITERATIONS, |iteration| {
            core_inline.write(BASE_VALUE ^ iteration as u32);
            core_inline.check()
        }),
    ];
    print_results("Direct core / InlineStore", &inline_results);

    let serialized = SerializedHardened::<u32, Tmr, _, 3>::new(BASE_VALUE, InlineStore::new(0));
    let serialized_results = [
        measure("serialized read_checked()", READ_ITERATIONS, |_| {
            serialized.read_checked()
        }),
        measure("serialized check()", CHECK_ITERATIONS, |_| {
            serialized.check()
        }),
        measure("serialized write()", WRITE_ITERATIONS, |iteration| {
            serialized.write(BASE_VALUE ^ iteration as u32);
            serialized.check()
        }),
    ];
    print_results("Serialized wrapper / InlineStore", &serialized_results);

    let critical = CriticalSectionHardened::<_, u32, Tmr, _, 3>::new(
        NoopCriticalSection,
        BASE_VALUE,
        InlineStore::new(0),
    );
    let critical_results = [
        measure("critical read_checked()", READ_ITERATIONS, |_| {
            critical.read_checked()
        }),
        measure("critical check()", CHECK_ITERATIONS, |_| critical.check()),
        measure("critical write()", WRITE_ITERATIONS, |iteration| {
            critical.write(BASE_VALUE ^ iteration as u32);
            critical.check()
        }),
    ];
    print_results("Critical-section wrapper / InlineStore", &critical_results);

    let mut split = Hardened::<u32, Tmr, _, 3>::new(BASE_VALUE, SplitStore::new(0, placement()));
    let split_results = [
        measure("split read_checked()", READ_ITERATIONS, |_| {
            split.read_checked()
        }),
        measure("split check()", CHECK_ITERATIONS, |_| split.check()),
        measure("split write()", WRITE_ITERATIONS, |iteration| {
            split.write(BASE_VALUE ^ iteration as u32);
            split.check()
        }),
    ];
    print_results("Direct core / SplitStore", &split_results);

    let mut complemented =
        Hardened::<u32, Tmr, _, 3>::new(BASE_VALUE, ComplementedStore::new(0, placement()));
    let complemented_results = [
        measure("complemented read_checked()", READ_ITERATIONS, |_| {
            complemented.read_checked()
        }),
        measure("complemented check()", CHECK_ITERATIONS, |_| {
            complemented.check()
        }),
        measure("complemented write()", WRITE_ITERATIONS, |iteration| {
            complemented.write(BASE_VALUE ^ iteration as u32);
            complemented.check()
        }),
    ];
    print_results("Direct core / ComplementedStore", &complemented_results);

    let repair_store = SharedStore::<u32, 3>::new(0);
    let mut repair_core = Hardened::<u32, Tmr, _, 3>::new(BASE_VALUE, repair_store.clone());
    let serialized_store = SharedStore::<u32, 3>::new(0);
    let serialized_repair =
        SerializedHardened::<u32, Tmr, _, 3>::new(BASE_VALUE, serialized_store.clone());
    let critical_store = SharedStore::<u32, 3>::new(0);
    let critical_repair = CriticalSectionHardened::<_, u32, Tmr, _, 3>::new(
        NoopCriticalSection,
        BASE_VALUE,
        critical_store.clone(),
    );
    let split_store = SplitStore::new(0, placement());
    let mut split_repair = Hardened::<u32, Tmr, _, 3>::new(BASE_VALUE, &split_store);
    let complemented_store = ComplementedStore::new(0, placement());
    let mut complemented_repair = Hardened::<u32, Tmr, _, 3>::new(BASE_VALUE, &complemented_store);

    let repair_results = [
        measure("core repair()", REPAIR_ITERATIONS, |iteration| {
            let slot = iteration % 3;
            let corrupted = BASE_VALUE ^ ((iteration as u32 & 0xFF) + 1);
            corrupt_slot(&repair_store, slot, corrupted).expect("valid repair slot");
            repair_core.repair()
        }),
        measure("serialized repair()", REPAIR_ITERATIONS, |iteration| {
            let slot = iteration % 3;
            let corrupted = BASE_VALUE ^ ((iteration as u32 & 0xFF) + 1);
            corrupt_slot(&serialized_store, slot, corrupted).expect("valid repair slot");
            serialized_repair.repair()
        }),
        measure("critical repair()", REPAIR_ITERATIONS, |iteration| {
            let slot = iteration % 3;
            let corrupted = BASE_VALUE ^ ((iteration as u32 & 0xFF) + 1);
            corrupt_slot(&critical_store, slot, corrupted).expect("valid repair slot");
            critical_repair.repair()
        }),
        measure("split repair()", REPAIR_ITERATIONS, |iteration| {
            let slot = iteration % 3;
            let corrupted = BASE_VALUE ^ ((iteration as u32 & 0xFF) + 1);
            split_store.write_slot(slot, corrupted);
            split_repair.repair()
        }),
        measure("complemented repair()", REPAIR_ITERATIONS, |iteration| {
            let slot = iteration % 3;
            let corrupted = BASE_VALUE ^ ((iteration as u32 & 0xFF) + 1);
            complemented_store.write_slot(slot, corrupted);
            complemented_repair.repair()
        }),
    ];
    print_results(
        "Repair after injected single-slot corruption",
        &repair_results,
    );

    println!("\nStorage footprint (bytes)");
    println!(
        "- {:34} {}",
        "InlineStore<u32, 3>",
        size_of::<InlineStore<u32, 3>>()
    );
    println!(
        "- {:34} {}",
        "SplitStore<u32, 3>",
        size_of::<SplitStore<u32, 3>>()
    );
    println!(
        "- {:34} {}",
        "ComplementedStore<u32, 3>",
        size_of::<ComplementedStore<u32, 3>>()
    );
    println!(
        "- {:34} {}",
        "Hardened<u32, Tmr, InlineStore>",
        size_of::<Hardened<u32, Tmr, InlineStore<u32, 3>, 3>>()
    );
    println!(
        "- {:34} {}",
        "SerializedHardened<u32, Tmr, InlineStore>",
        size_of::<SerializedHardened<u32, Tmr, InlineStore<u32, 3>, 3>>()
    );
    println!(
        "- {:34} {}",
        "CriticalSectionHardened<u32, Tmr, InlineStore>",
        size_of::<CriticalSectionHardened<NoopCriticalSection, u32, Tmr, InlineStore<u32, 3>, 3>>()
    );
}
