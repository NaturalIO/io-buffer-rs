//! # io-buffer
//!
//! This crate provide a [Buffer] type, to unify the difference of different types of buffer,
//! for disk and network IO:
//!
//! * Converts owned buffer, `From<Vec<u8>>` and `To<Vec<u8>>`.
//!
//! * Allocation with [malloc()](Buffer::alloc())
//!
//! * Allocation with [posix_memalign()](Buffer::aligned())
//!
//! * Converts from [const reference](Buffer::from_c_ref_const()),  or from
//! [mutable reference](Buffer::from_c_ref_mut()) of unsafe c code.
//!
//! On debug mode, provides runtime checking if you try to as_mut() a const buffer.
//!
//! ## Usage
//!
//! Cargo.toml:
//!
//! ``` toml
//! [dependencies]
//! io-buffer = "1"
//! ```
//!
//! ## Feature flags
//!
//! * compress: enable [Compression] trait
//!
//! * lz4: enable lz4 compression

extern crate log;
#[macro_use]
extern crate captains_log;

mod buffer;
mod utils;

pub use buffer::{Buffer, MAX_BUFFER_SIZE};
pub use utils::*;

#[cfg(any(feature = "compress", doc))]
/// Enabled with feature `compress`
pub mod compress;

#[cfg(test)]
mod test;
