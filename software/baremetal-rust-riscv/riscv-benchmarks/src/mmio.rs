//! MMIO (Memory-Mapped I/O) utilities
//! 
//! This module provides safe wrappers around volatile memory operations
//! for interacting with memory-mapped hardware registers.

use core::ptr::{read_volatile, write_volatile};

#[inline(always)]
pub unsafe fn reg_write8(addr: usize, data: u8) {
    let ptr = addr as *mut u8;
    write_volatile(ptr, data);
}

#[inline(always)]
pub unsafe fn reg_read8(addr: usize) -> u8 {
    let ptr = addr as *const u8;
    read_volatile(ptr)
}

#[inline(always)]
pub unsafe fn reg_write16(addr: usize, data: u16) {
    let ptr = addr as *mut u16;
    write_volatile(ptr, data);
}

#[inline(always)]
pub unsafe fn reg_read16(addr: usize) -> u16 {
    let ptr = addr as *const u16;
    read_volatile(ptr)
}

#[inline(always)]
pub unsafe fn reg_write32(addr: usize, data: u32) {
    let ptr = addr as *mut u32;
    write_volatile(ptr, data);
}

#[inline(always)]
pub unsafe fn reg_read32(addr: usize) -> u32 {
    let ptr = addr as *const u32;
    read_volatile(ptr)
}

#[inline(always)]
pub unsafe fn reg_write64(addr: usize, data: u64) {
    let ptr = addr as *mut u64;
    write_volatile(ptr, data);
}

#[inline(always)]
pub unsafe fn reg_read64(addr: usize) -> u64 {
    let ptr = addr as *const u64;
    read_volatile(ptr)
}
