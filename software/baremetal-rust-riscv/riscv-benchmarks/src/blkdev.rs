//! Block Device (blkdev) driver
//! 
//! This module provides a Rust interface for interacting with the block device
//! hardware via memory-mapped I/O registers.

use crate::mmio::{reg_read8, reg_read32, reg_write8, reg_write32, reg_write64};
use core::arch::asm;

// Block device register addresses
pub const BLKDEV_BASE: usize = 0x10015000;
pub const BLKDEV_ADDR: usize = BLKDEV_BASE;
pub const BLKDEV_OFFSET: usize = BLKDEV_BASE + 8;
pub const BLKDEV_LEN: usize = BLKDEV_BASE + 12;
pub const BLKDEV_WRITE: usize = BLKDEV_BASE + 16;
pub const BLKDEV_REQUEST: usize = BLKDEV_BASE + 17;
pub const BLKDEV_NREQUEST: usize = BLKDEV_BASE + 18;
pub const BLKDEV_COMPLETE: usize = BLKDEV_BASE + 19;
pub const BLKDEV_NCOMPLETE: usize = BLKDEV_BASE + 20;
pub const BLKDEV_NSECTORS: usize = BLKDEV_BASE + 24;
pub const BLKDEV_MAX_REQUEST_LENGTH: usize = BLKDEV_BASE + 28;

pub const BLKDEV_SECTOR_SIZE: usize = 512;
pub const BLKDEV_SECTOR_SHIFT: usize = 9;

#[inline(always)]
pub fn blkdev_nsectors() -> u32 {
    unsafe { reg_read32(BLKDEV_NSECTORS) }
}

#[inline(always)]
pub fn blkdev_max_req_len() -> u32 {
    unsafe { reg_read32(BLKDEV_MAX_REQUEST_LENGTH) }
}
#[inline(always)]
pub fn blkdev_send_request(addr: usize, offset: u32, len: u32, write: bool) -> u8 {
    unsafe {
        reg_write64(BLKDEV_ADDR, addr as u64);
        reg_write32(BLKDEV_OFFSET, offset);
        reg_write32(BLKDEV_LEN, len);
        reg_write8(BLKDEV_WRITE, if write { 1 } else { 0 });

        asm!("fence");
        reg_read8(BLKDEV_REQUEST)
    }
}
