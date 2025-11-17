#![no_main]
#![no_std]

use riscv_benchmarks::*;
use riscv_rt::entry;
use md5::{Md5, Digest};
use md5::digest::generic_array::{typenum, GenericArray};
use core::hint::black_box;

const MSG_SIZE: usize = 10;
const SCALE_FACTOR: usize = 10;

fn compute_hash(data: [u8; MSG_SIZE]) -> GenericArray<u8, typenum::U16> {
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize()
}

#[entry]
fn main() -> ! {
    let mut data: [u8; MSG_SIZE] = [0; MSG_SIZE];
    for (i, x) in data.iter_mut().enumerate() {
        *x = i as u8;
    }
    for _ in 0..SCALE_FACTOR {
        let benchmark_data = start_benchmark();
        let x = black_box(compute_hash(black_box(data)));
        print_benchmark_data(benchmark_data);
    }
    exit();
}
