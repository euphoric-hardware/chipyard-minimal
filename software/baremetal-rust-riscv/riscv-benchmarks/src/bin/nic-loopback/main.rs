#![no_main]
#![no_std]

use core::fmt::Write;
use htif::HostFile;
use riscv::register;
use riscv_benchmarks::{exit, mmio::*, nic::*};
use riscv_rt::entry;

const NPACKETS: usize = 10;
const TEST_OFFSET: usize = 3;
const TEST_LEN: usize = 356;
const ARRAY_LEN: usize = 360;
const NTRIALS: usize = 3;

static mut SRC: [[u32; ARRAY_LEN]; NPACKETS] = [[0; ARRAY_LEN]; NPACKETS];
static mut DST: [[u32; ARRAY_LEN]; NPACKETS] = [[0; ARRAY_LEN]; NPACKETS];
static mut LENGTHS: [u64; NPACKETS] = [0; NPACKETS];

unsafe fn send_recv() {
    let mut send_comps_left = NPACKETS;
    let mut recv_comps_left = NPACKETS;
    let mut recv_idx = 0;

    for i in 0..NPACKETS {
        let pkt_size = TEST_LEN * core::mem::size_of::<u32>();
        let src_addr = SRC[i].as_ptr().add(TEST_OFFSET) as usize;
        let send_packet = ((pkt_size as u64) << 48) | (src_addr as u64);
        let recv_addr = DST[i].as_ptr() as u64;

        reg_write64(SIMPLENIC_SEND_REQ, send_packet);
        reg_write64(SIMPLENIC_RECV_REQ, recv_addr);
    }

    while send_comps_left > 0 || recv_comps_left > 0 {
        let ncomps = nic_send_comp_avail();
        core::arch::asm!("fence");
        for _ in 0..ncomps {
            reg_read16(SIMPLENIC_SEND_COMP);
        }
        send_comps_left -= ncomps as usize;

        let ncomps = nic_recv_comp_avail();
        core::arch::asm!("fence");
        for _ in 0..ncomps {
            LENGTHS[recv_idx] = reg_read16(SIMPLENIC_RECV_COMP) as u64;
            recv_idx += 1;
        }
        recv_comps_left -= ncomps as usize;
    }
}

unsafe fn run_test() {
    for i in 0..NPACKETS {
        for j in 0..ARRAY_LEN {
            DST[i][j] = 0;
        }
    }
    core::arch::asm!("fence");

    let start = register::mcycle::read();
    send_recv();
    let end = register::mcycle::read();

    writeln!(HostFile::stdout(), "send/recv {} cycles", end - start).unwrap();

    let expected_len = (TEST_LEN * core::mem::size_of::<u32>()) as u64;
    for i in 0..NPACKETS {
        if LENGTHS[i] != expected_len {
            writeln!(
                HostFile::stdout(),
                "recv got wrong # bytes: expected {}, got {}",
                expected_len,
                LENGTHS[i]
            )
            .unwrap();
            htif::exit(1);
        }

        for j in 0..TEST_LEN {
            if DST[i][j] != SRC[i][j + TEST_OFFSET] {
                writeln!(
                    HostFile::stdout(),
                    "Data mismatch @ {}, {}: {:x} != {:x}",
                    i,
                    j,
                    DST[i][j],
                    SRC[i][j + TEST_OFFSET]
                )
                .unwrap();
                htif::exit(1);
            }
        }
    }
}

#[entry]
fn main() -> ! {
    unsafe {
        for i in 0..NPACKETS {
            for j in 0..ARRAY_LEN {
                SRC[i][j] = (i * ARRAY_LEN + j) as u32;
            }
        }

        for i in 0..NTRIALS {
            writeln!(HostFile::stdout(), "Trial {}", i).unwrap();
            run_test();
        }

        writeln!(HostFile::stdout(), "All correct").unwrap();
    }

    exit()
}
