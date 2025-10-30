#![no_main]
#![no_std]

use core::fmt::Write;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use htif::HostFile;
use riscv::register;
use riscv_benchmarks::{exit, marchid};
use riscv_rt::entry;

// EDIT THIS: Number of cores in the system
const N_CORES: usize = 4;

// Wrapper to make UnsafeCell<bool> Sync for use in static context
// SAFETY: Each hart only accesses its own index in the array
struct SyncUnsafeCell<T>(UnsafeCell<T>);
unsafe impl<T> Sync for SyncUnsafeCell<T> {}

impl<T> SyncUnsafeCell<T> {
    const fn new(value: T) -> Self {
        SyncUnsafeCell(UnsafeCell::new(value))
    }
    
    fn get(&self) -> *mut T {
        self.0.get()
    }
}

// Per-hart storage for barrier sense (indexed by hart ID)
static THREAD_SENSE: [SyncUnsafeCell<bool>; N_CORES] = [
    SyncUnsafeCell::new(false),
    SyncUnsafeCell::new(false),
    SyncUnsafeCell::new(false),
    SyncUnsafeCell::new(false),
];

// Static variables for barrier synchronization
static BARRIER_COUNT: AtomicUsize = AtomicUsize::new(0);
static BARRIER_SENSE: AtomicBool = AtomicBool::new(false);

/// A barrier synchronization primitive for multiple cores
#[inline(never)]
fn barrier(hartid: usize) {
    unsafe {
        // Ensure all previous memory operations complete
        core::sync::atomic::fence(Ordering::SeqCst);

        // Toggle thread's sense
        let sense_ptr = THREAD_SENSE[hartid].get();
        *sense_ptr = !*sense_ptr;
        let my_sense = *sense_ptr;

        // Increment the counter
        let count = BARRIER_COUNT.fetch_add(1, Ordering::SeqCst);

        if count == N_CORES - 1 {
            // Last thread to arrive resets the counter and toggles the sense
            BARRIER_COUNT.store(0, Ordering::SeqCst);
            BARRIER_SENSE.store(my_sense, Ordering::SeqCst);
        } else {
            // Wait for the sense to change
            while BARRIER_SENSE.load(Ordering::SeqCst) != my_sense {
                // Spin wait
                core::hint::spin_loop();
            }
        }

        // Ensure all subsequent memory operations happen after barrier
        core::sync::atomic::fence(Ordering::SeqCst);
    }
}

fn __main() {
    let mhartid = register::mhartid::read();

    // If hart ID is beyond the number of cores we expect, spin forever
    if mhartid >= N_CORES {
        loop {
            core::hint::spin_loop();
        }
    }

    // Read marchid CSR and extract the value
    let marchid_val = register::marchid::read()
        .map(|m| m.bits())
        .unwrap_or(0);
    let march = marchid::get_march(marchid_val);

    // Each core prints its message in sequence
    for i in 0..N_CORES {
        if mhartid == i {
            writeln!(
                HostFile::stdout(),
                "Hello world from core {}, a {}",
                mhartid, march
            )
            .ok();
        }
        barrier(mhartid);
    }

    // Non-zero harts spin forever
    if mhartid > 0 {
        loop {
            core::hint::spin_loop();
        }
    }
}

#[entry]
fn main() -> ! {
    __main();
    exit()
}
