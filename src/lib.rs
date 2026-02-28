#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(docsrs, allow(unused_attributes))]
#![doc = include_str!("../README.md")]

mod buffer;
mod utils;

pub use buffer::{Buffer, MAX_BUFFER_SIZE};
pub use utils::*;

#[cfg(any(feature = "compress", doc))]
/// Enabled with feature `compress`
pub mod compress;

#[cfg(test)]
mod test;
