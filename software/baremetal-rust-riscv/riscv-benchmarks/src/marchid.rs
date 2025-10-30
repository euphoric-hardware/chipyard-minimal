//! RISC-V Architecture ID (marchid) utilities
//! 
//! This module provides functions to map marchid CSR values to 
//! human-readable architecture names.

/// Get the architecture name from the marchid CSR value
/// 
/// # Arguments
/// * `marchid` - The value from the marchid CSR register
/// 
/// # Returns
/// A string slice with the architecture name
pub fn get_march(marchid: usize) -> &'static str {
    match marchid {
        1 => "rocket",
        2 => "sonicboom",
        5 => "spike",
        _ => "unknown",
    }
}
