# io-buffer

[![Build Status](https://github.com/NaturalIO/io-buffer-rs/workflows/Rust/badge.svg)](
https://github.com/NaturalIO/io-buffer-rs/actions)
[![Cargo](https://img.shields.io/crates/v/io-buffer.svg)](
https://crates.io/crates/io-buffer)
[![Documentation](https://docs.rs/io-buffer/badge.svg)](
https://docs.rs/io-buffer)
[![Rust 1.36+](https://img.shields.io/badge/rust-1.36+-lightgray.svg)](
https://www.rust-lang.org)


This crate provide a [Buffer] type, to unify the difference of different types of buffer,
for disk and network IO:

* Converts owned buffer, `From<Vec<u8>>` and `To<Vec<u8>>`.

* Allocation with `malloc()`(Buffer::alloc())

* Allocation with `posix_memalign()`(Buffer::aligned())

* Converts from const reference (Buffer::from_c_ref_const()),  or from
mutable reference (Buffer::from_c_ref_mut()) of unsafe c code.

On debug mode, provides runtime checking if you try to as_mut() a const buffer.
