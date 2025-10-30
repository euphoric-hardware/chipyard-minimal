#![no_main]
#![no_std]

use core::fmt::Write;
use htif::HostFile;
use riscv_benchmarks::exit;
use riscv_rt::entry;

#[entry]
fn main() -> ! {
    writeln!(HostFile::stdout(), "Hello World");
    exit()
}

