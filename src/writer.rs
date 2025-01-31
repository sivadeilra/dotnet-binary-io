use std::io::Write;
use zerocopy::IntoBytes;

extern crate alloc;
use alloc::vec::Vec;

pub type Result<T> = core::result::Result<T, BinaryWriterError>;

/// Encodes binary values, using the same rules as .NET's `System.IO.BinaryWriter`.
pub struct BinaryWriter<T> {
    /// The output data.
    pub out: T,
}

impl<T: Write> BinaryWriter<T> {
    /// Constructor
    pub fn wrap(out: T) -> Self {
        Self { out }
    }

    /// Extracts the inner buffer
    pub fn into_inner(self) -> T {
        self.out
    }

    /// Accesses the inner buffer
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.out
    }
}

impl BinaryWriter<Vec<u8>> {
    /// Creates a new `BinaryWriter` over a `Vec<u8>`
    pub fn new() -> Self {
        Self { out: Vec::new() }
    }

    /// Creates a new `BinaryWriter` over a `Vec<u8>` with the given capacity.
    pub fn with_capacity(len: usize) -> Self {
        Self {
            out: Vec::with_capacity(len),
        }
    }

    /// Writes `bytes` to the output.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        self.out.extend_from_slice(bytes);
    }

    /// Writes a small, fixed-size array of bytes.
    pub fn write_cbytes<const N: usize>(&mut self, value: [u8; N]) {
        self.write_bytes(&value)
    }

    /// Writes a single `u8` value
    pub fn write_u8(&mut self, value: u8) {
        self.write_bytes(&[value])
    }

    /// Writes a single `i8` value
    pub fn write_i8(&mut self, value: i8) {
        self.write_bytes(&[value as u8])
    }

    /// Writes a single `u16` value
    pub fn write_u16(&mut self, value: u16) {
        self.write_cbytes(value.to_le_bytes())
    }

    /// Writes a single `u32` value
    pub fn write_u32(&mut self, value: u32) {
        self.write_cbytes(value.to_le_bytes())
    }

    /// Writes a single `u64` value
    pub fn write_u64(&mut self, value: u64) {
        self.write_cbytes(value.to_le_bytes())
    }

    /// Writes a single `i16` value
    pub fn write_i16(&mut self, value: i16) {
        self.write_cbytes(value.to_le_bytes())
    }

    /// Writes a single `i32` value
    pub fn write_i32(&mut self, value: i32) {
        self.write_cbytes(value.to_le_bytes())
    }

    /// Writes a single `i64` value
    pub fn write_i64(&mut self, value: i64) {
        self.write_cbytes(value.to_le_bytes())
    }

    /// Encodes an `i32` value using a variable-length encoding.
    ///
    /// Although this function takes `i32` values, applications should avoid using this for
    /// negative values. This function can correctly encode negative values, but most "small"
    /// negative value (e.g. `-10`) will be encoded with the maximum number of bytes, which wastes
    /// space.
    pub fn write_7bit_encoded_i32(&mut self, value: i32) {
        const MORE: u8 = 0x80; // bit indicating there are more bits
        const MASK: u8 = 0x7f;

        let w0: u8 = value as u8 & MASK; // 7 significant bits
        let w1: u8 = (value >> 7) as u8 & MASK; // 7 significant bits
        let w2: u8 = (value >> 14) as u8 & MASK; // 7 significant bits
        let w3: u8 = (value >> 21) as u8 & MASK; // 7 significant bits
        let w4: u8 = (value >> 28) as u8 & 0xF; // only 4 significant bits

        if w4 != 0 {
            self.write_cbytes([w0 | MORE, w1 | MORE, w2 | MORE, w3 | MORE, w4]);
        } else if w3 != 0 {
            self.write_cbytes([w0 | MORE, w1 | MORE, w2 | MORE, w3]);
        } else if w2 != 0 {
            self.write_cbytes([w0 | MORE, w1 | MORE, w2]);
        } else if w1 != 0 {
            self.write_cbytes([w0 | MORE, w1]);
        } else {
            self.write_cbytes([w0]);
        }
    }

    /// Encodes an `i64` value using a variable-length encoding.
    ///
    /// Although this function takes `i64` values, applications should avoid using this for
    /// negative values. This function can correctly encode negative values, but most "small"
    /// negative value (e.g. `-10`) will be encoded with the maximum number of bytes, which wastes
    /// space.
    pub fn write_7bit_encoded_i64(&mut self, value: i64) {
        let mut n: u64 = value as u64;

        loop {
            if n < 0x80 {
                self.write_u8(n as u8);
                break;
            }
            self.write_u8((n & 0x7f) as u8 | 0x80);
            n >>= 7;
        }
    }

    /// Writes a `bool` value. True is encoded as 1. False is encoded as 0.
    pub fn write_bool(&mut self, value: bool) {
        self.write_u8(value as u8)
    }

    /// Writes an `f32` value. The value is encoded using its 4-byte little-endian in-memory
    /// representation.
    pub fn write_f32(&mut self, value: f32) {
        self.write_cbytes(value.to_le_bytes());
    }

    /// Writes an `f64` value. The value is encoded using its 4-byte little-endian in-memory
    /// representation.
    pub fn write_f64(&mut self, value: f64) {
        self.write_cbytes(value.to_le_bytes());
    }

    /// Writes a UTF-8 string in length-prefixed form.
    pub fn write_utf8_str(&mut self, s: &str) -> Result<()> {
        let len_i32 = i32::try_from(s.len()).map_err(|_| BinaryWriterError::CannotEncode)?;
        self.write_7bit_encoded_i32(len_i32);
        self.write_bytes(s.as_bytes());
        Ok(())
    }

    /// Writes a UTF-8 string in length-prefixed form.
    ///
    /// This function does not validate that the input string is well-formed UTF-8.
    pub fn write_utf8_bytes(&mut self, s: &[u8]) -> Result<()> {
        let len_i32 = i32::try_from(s.len()).map_err(|_| BinaryWriterError::CannotEncode)?;
        self.write_7bit_encoded_i32(len_i32);
        self.write_bytes(s);
        Ok(())
    }

    /// Writes a UTF-16 string in length-prefixed form.
    ///
    /// This function does not validate that the input string is well-formed UTF-16.
    pub fn write_utf16_wchars(&mut self, s: &[u16]) -> Result<()> {
        let s_bytes = s.as_bytes();
        let len_i32 = i32::try_from(s_bytes.len()).map_err(|_| BinaryWriterError::CannotEncode)?;
        self.write_7bit_encoded_i32(len_i32);
        self.write_bytes(s_bytes);
        Ok(())
    }

    /// Converts a UTF-8 string into UTF-16 and writes it in length-prefixed form.
    pub fn write_utf16_encode(&mut self, s: &str) {
        let num_utf16_code_units = s.encode_utf16().count();
        let len_bytes: usize = num_utf16_code_units * 2;
        self.write_7bit_encoded_i32(len_bytes as i32);

        self.out.reserve(len_bytes);
        for c in s.encode_utf16() {
            self.write_u16(c);
        }
    }
}

/// Error type for some `write_*` functions of `BinaryWriter`.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum BinaryWriterError {
    /// Indicates that a value cannot be encoded. This is used for cases where a string or slice
    /// is too large to encode using the variable-length encoding rules.
    CannotEncode,
}

impl core::error::Error for BinaryWriterError {}

impl core::fmt::Display for BinaryWriterError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::CannotEncode => f.write_str("The data cannot be encoded"),
        }
    }
}
