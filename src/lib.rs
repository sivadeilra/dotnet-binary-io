//! Reads and writes buffers using the same encoding rules as .NET's `System.IO.BinaryWriter`.
//!
//! # References
//! * <https://learn.microsoft.com/en-us/dotnet/api/system.io.binarywriter.write?view=net-9.0>

#![cfg_attr(not(any(feature = "std", test)), no_std)]
#![forbid(unsafe_code)]
#![forbid(unused_must_use)]
#![warn(missing_docs)]

mod reader;
mod writer;

#[cfg(test)]
mod tests;

pub use reader::{BinaryReader, ReaderError};
pub use writer::BinaryWriter;
