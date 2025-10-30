#![no_main]
#![no_std]

use core::fmt::Write;
use core::arch::asm;
use htif::HostFile;
use riscv_benchmarks::{exit, blkdev::{self, *}, mmio::{reg_read8, reg_write8}};
use riscv_rt::entry;

const TEST_NSECTORS: u32 = 4;
const TEST_SIZE: usize = (TEST_NSECTORS as usize * BLKDEV_SECTOR_SIZE) / core::mem::size_of::<u32>();

#[repr(align(64))]
struct AlignedBuffer([u32; TEST_SIZE]);

static mut TEST_DATA: AlignedBuffer = AlignedBuffer([0; TEST_SIZE]);
static mut RES_DATA: AlignedBuffer = AlignedBuffer([0; TEST_SIZE]);

fn blkdev_read(addr: &mut [u32], offset: u32, nsectors: u32) {
    let ntags = unsafe { reg_read8(BLKDEV_NREQUEST) } as usize;
    let nsectors_per_tag = nsectors / ntags as u32;

    writeln!(HostFile::stdout(), "sending {} reads", ntags).ok();

    let mut current_offset = offset;
    let chunk_size = nsectors_per_tag as usize * BLKDEV_SECTOR_SIZE / core::mem::size_of::<u32>();

    for i in 0..ntags {
        let chunk_addr = unsafe { 
            addr.as_mut_ptr().add(i * chunk_size) as usize
        };

        blkdev::blkdev_send_request(
            chunk_addr,
            current_offset,
            nsectors_per_tag,
            false
        );
        current_offset += nsectors_per_tag;
    }

    while unsafe { reg_read8(BLKDEV_NCOMPLETE) } < ntags as u8 {}

    for _ in 0..ntags {
        let resp_tag = unsafe { reg_read8(BLKDEV_COMPLETE) };
        writeln!(HostFile::stdout(), "completed read {}", resp_tag).ok();
    }
}

fn blkdev_write(offset: u32, addr: &[u32], nsectors: u32) {
    let ntags = unsafe { reg_read8(BLKDEV_NREQUEST) } as usize;
    let nsectors_per_tag = nsectors / ntags as u32;

    writeln!(HostFile::stdout(), "sending {} writes", ntags).ok();

    let mut current_offset = offset;
    let chunk_size = nsectors_per_tag as usize * BLKDEV_SECTOR_SIZE / core::mem::size_of::<u32>();

    for i in 0..ntags {
        let chunk_addr = unsafe { 
            addr.as_ptr().add(i * chunk_size) as usize
        };

        blkdev::blkdev_send_request(
            chunk_addr,
            current_offset,
            nsectors_per_tag,
            true
        );
        current_offset += nsectors_per_tag;
    }

    while unsafe { reg_read8(BLKDEV_NCOMPLETE) } < ntags as u8 {}

    for _ in 0..ntags {
        let resp_tag = unsafe { reg_read8(BLKDEV_COMPLETE) };
        writeln!(HostFile::stdout(), "completed write {}", resp_tag).ok();
    }
}

#[entry]
fn main() -> ! {
    let nsectors = blkdev::blkdev_nsectors();
    let max_req_len = blkdev::blkdev_max_req_len();

    if nsectors < TEST_NSECTORS {
        writeln!(
            HostFile::stdout(),
            "Error: blkdev nsectors not large enough: {} < {}",
            nsectors, TEST_NSECTORS
        ).ok();
        htif::htif_fail(1);
    }

    if max_req_len < TEST_NSECTORS {
        writeln!(
            HostFile::stdout(),
            "Error: blkdev max_req_len not large enough: {} < {}",
            max_req_len, TEST_NSECTORS
        ).ok();
        htif::htif_fail(1);
    }

    writeln!(
        HostFile::stdout(),
        "blkdev: {} sectors {} max request length",
        nsectors, max_req_len
    ).ok();

    unsafe {
        for i in 0..TEST_SIZE {
            TEST_DATA.0[i] = (i << 8) as u32;
        }

        asm!("fence");

        blkdev_write(0, &TEST_DATA.0, TEST_NSECTORS);
        blkdev_read(&mut RES_DATA.0, 0, TEST_NSECTORS);

        for i in 0..TEST_SIZE {
            if TEST_DATA.0[i] != RES_DATA.0[i] {
                writeln!(
                    HostFile::stdout(),
                    "data mismatch at {}: {:x} != {:x}",
                    i, TEST_DATA.0[i], RES_DATA.0[i]
                ).ok();
                htif::htif_fail(1);
            }
        }
    }
    writeln!(HostFile::stdout(), "All correct").ok();
    exit()
}
