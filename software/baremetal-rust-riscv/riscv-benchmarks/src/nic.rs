//! SimpleNIC driver
//!
//! This module provides a driver interface for the SimpleNIC network interface.
//! The NIC supports sending and receiving packets through memory-mapped registers.

use crate::mmio::*;

pub const SIMPLENIC_BASE: usize = 0x10016000;

pub const SIMPLENIC_SEND_REQ: usize = SIMPLENIC_BASE + 0;
pub const SIMPLENIC_RECV_REQ: usize = SIMPLENIC_BASE + 8;
pub const SIMPLENIC_SEND_COMP: usize = SIMPLENIC_BASE + 16;
pub const SIMPLENIC_RECV_COMP: usize = SIMPLENIC_BASE + 18;
pub const SIMPLENIC_COUNTS: usize = SIMPLENIC_BASE + 20;
pub const SIMPLENIC_MACADDR: usize = SIMPLENIC_BASE + 24;

#[inline]
pub fn nic_send_req_avail() -> u32 {
    unsafe { reg_read32(SIMPLENIC_COUNTS) & 0xff }
}

#[inline]
pub fn nic_recv_req_avail() -> u32 {
    unsafe { (reg_read32(SIMPLENIC_COUNTS) >> 8) & 0xff }
}

#[inline]
pub fn nic_send_comp_avail() -> u32 {
    unsafe { (reg_read32(SIMPLENIC_COUNTS) >> 16) & 0xff }
}

#[inline]
pub fn nic_recv_comp_avail() -> u32 {
    unsafe { (reg_read32(SIMPLENIC_COUNTS) >> 24) & 0xff }
}

pub unsafe fn nic_send(data: *const u8, len: usize) {
    let addr = (data as usize) & ((1 << 48) - 1);
    let packet = ((len as u64) << 48) | (addr as u64);

    while nic_send_req_avail() == 0 {}
    reg_write64(SIMPLENIC_SEND_REQ, packet);

    while nic_send_comp_avail() == 0 {}
    reg_read16(SIMPLENIC_SEND_COMP);
}

pub unsafe fn nic_recv(dest: *mut u8) -> u16 {
    let addr = dest as usize;

    while nic_recv_req_avail() == 0 {}
    reg_write64(SIMPLENIC_RECV_REQ, addr as u64);

    while nic_recv_comp_avail() == 0 {}
    let len = reg_read16(SIMPLENIC_RECV_COMP);

    core::arch::asm!("fence");

    len
}

#[inline]
pub fn nic_macaddr() -> u64 {
    unsafe { reg_read64(SIMPLENIC_MACADDR) }
}
