#[cfg(feature = "std")]
use std::borrow::Cow;

use zerocopy::byteorder::{LE, U16};
use zerocopy::FromBytes;

pub type Result<T> = core::result::Result<T, ReaderError>;

/// Reads values from a slice of bytes. The values are encoded using the rules defined by .NET's
/// `System.IO.BinaryWriter`.
///
/// Most simple fixed-size types are simply encoded using in-memory byte representation of the type,
/// using little-endian memory ordering if applicable.
///
/// Variable-length types, such as strings and variable-length integers, have different encodings.
/// Each of the methods that decodes such a type describes its representation.
///
/// This type only supports reading values from a slice of bytes. If you need to read values from
/// a file or `Read` implementation, then you should copy the data into an in-memory buffer first.
///
/// Another option is to use "restartable" decoding.  Before calling any function that decodes a
/// value, read the `data` slice (or simply its length). Then, call a function to decode a value
/// (potentially multiple calls to decode multiple values).  If any function fails with
/// `Err(ReaderError::NeedsMoreData)`, then go read more data from the source and reset `data`
/// to point to the original location, plus any new data. Then repeat the calls that decode data.
///
/// This is feasible and it may be necessary for some designs. However, simply reading data into
/// `Vec<u8>` or another in-memory container is likely to be simpler, less bug-prone, and
/// probably faster, too.
pub struct BinaryReader<'a> {
    /// The input data being parsed. Each time a value is parsed from `data`, `data` is reassigned
    /// to the remaining data.
    pub data: &'a [u8],
}

impl<'a> BinaryReader<'a> {
    /// Constructor
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Reads a single `u8` value.
    #[inline(always)]
    pub fn read_u8(&mut self) -> Result<u8> {
        if !self.data.is_empty() {
            let value = self.data[0];
            self.data = &self.data[1..];
            Ok(value)
        } else {
            Err(ReaderError::NeedsMoreData)
        }
    }

    /// Reads a slice of bytes whose length is `len`. This function returns a slice reference
    /// to the bytes; it does not copy them.
    #[inline(always)]
    pub fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.data.len() < len {
            Err(ReaderError::NeedsMoreData)
        } else {
            let (lo, hi) = self.data.split_at(len);
            self.data = hi;
            Ok(lo)
        }
    }

    /// Reads a small array of bytes, with a constant length.
    #[inline(always)]
    pub fn read_cbytes<const N: usize>(&mut self) -> Result<[u8; N]> {
        if self.data.len() < N {
            Err(ReaderError::NeedsMoreData)
        } else {
            let (lo, hi) = self.data.split_at(N);
            self.data = hi;
            // This unwrap() call will get optimized out.
            Ok(*<&[u8; N]>::try_from(lo).unwrap())
        }
    }

    /// Reads a `u16` in little-endian byte order.
    #[inline(always)]
    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.read_cbytes()?))
    }

    /// Reads a `u32` in little-endian byte order.
    #[inline(always)]
    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.read_cbytes()?))
    }

    /// Reads a `u64` in little-endian byte order.
    #[inline(always)]
    pub fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.read_cbytes()?))
    }

    /// Reads a `i16` in little-endian byte order.
    #[inline(always)]
    pub fn read_i16(&mut self) -> Result<i16> {
        Ok(i16::from_le_bytes(self.read_cbytes()?))
    }

    /// Reads a `i32` in little-endian byte order.
    #[inline(always)]
    pub fn read_i32(&mut self) -> Result<i32> {
        Ok(i32::from_le_bytes(self.read_cbytes()?))
    }

    /// Reads a `i64` in little-endian byte order.
    #[inline(always)]
    pub fn read_i64(&mut self) -> Result<i64> {
        Ok(i64::from_le_bytes(self.read_cbytes()?))
    }

    /// Reads a variable-length integer and returns the value in `i32`.
    pub fn read_7bit_encoded_i32(&mut self) -> Result<i32> {
        // Each byte encodes 7 bits of the integer and 1 bit indicating whether there are
        // more bytes following this one. Because 32 is not evenly divisible by 7, the last
        // byte has some meaningless bits in them. We could validate those bits (rejecting
        // input where the last byte contains non-zero meaningless bits), but that would be
        // stricter than the .NET implementation, so we do not.

        const MORE: u8 = 0x80;

        let mut shift: u32 = 0;
        let mut n: u32 = 0;

        loop {
            let b = self.read_u8()?;
            n |= ((b & 0x7f) as u32) << shift;

            if (b & MORE) == 0 {
                break;
            }

            shift += 7;
            if shift >= 32 {
                return Err(ReaderError::Invalid);
            }
        }

        Ok(n as i32)
    }

    /// Reads a variable-length integer and returns the value in `i64`.
    pub fn read_7bit_encoded_i64(&mut self) -> Result<i64> {
        const MORE: u8 = 0x80;

        let mut shift: u32 = 0;
        let mut n: u64 = 0;

        loop {
            let b = self.read_u8()?;
            n |= ((b & 0x7f) as u64) << shift;

            if (b & MORE) == 0 {
                break;
            }

            shift += 7;
            if shift >= 64 {
                return Err(ReaderError::Invalid);
            }
        }

        Ok(n as i64)
    }

    /// Reads a length-prefixed UTF-8 string.
    ///
    /// This does not copy any data. It reads the prefixed length, locates the contents of the
    /// string, then returns the string data as a `&[u8]`.
    ///
    /// The caller must handle validating that the string is well-formed UTF-8, if necessary.
    pub fn read_utf8_bytes(&mut self) -> Result<&'a [u8]> {
        let len_i32 = self.read_7bit_encoded_i32()?;
        let Ok(len_usize) = usize::try_from(len_i32) else {
            return Err(ReaderError::Invalid);
        };

        self.read_bytes(len_usize)
    }

    /// Reads a length-prefixed UTF-8 string.
    ///
    /// This does not copy any data. It reads the prefixed length, locates the contents of the
    /// string, then returns the string data as a `bstr::BStr`.
    ///
    /// The caller must handle validating that the string is well-formed UTF-8, if necessary.
    ///
    /// The encoded stream does not contain any information that distinguishes UTF-8 strings and
    /// UTF-16 strings, so applications will need to make sure that they call the correct
    /// `read_utf8_*` or `read_utf16_*` function.
    #[cfg(feature = "bstr")]
    pub fn read_utf8_bstr(&mut self) -> Result<&'a bstr::BStr> {
        Ok(bstr::BStr::new(self.read_utf8_bytes()?))
    }

    /// Reads a length-prefixed UTF-8 string and returns it as `&str`.
    ///
    /// This does not copy any data. It reads the prefixed length, locates the contents of the
    /// string, validates that the contents are well-formed UTF-8 and returns the string slice.
    ///
    /// The encoded stream does not contain any information that distinguishes UTF-8 strings and
    /// UTF-16 strings, so applications will need to make sure that they call the correct
    /// `read_utf8_*` or `read_utf16_*` function.
    pub fn read_utf8_str(&mut self) -> Result<&'a str> {
        let bytes = self.read_utf8_bytes()?;
        if let Ok(s) = core::str::from_utf8(bytes) {
            Ok(s)
        } else {
            Err(ReaderError::NeedsMoreData)
        }
    }

    /// Reads a length-prefixed UTF-8 string and returns it as `Cow<str>`.
    ///
    /// The input string is expected to be valid UTF-8. However, if the input contains byte
    /// sequences that do not code for valid UTF-8, then those sequences will be replaced with
    /// the Unicore replacement character and the rest of the string will be processed.
    ///
    /// The encoded stream does not contain any information that distinguishes UTF-8 strings and
    /// UTF-16 strings, so applications will need to make sure that they call the correct
    /// `read_utf8_*` or `read_utf16_*` function.
    #[cfg(feature = "std")]
    pub fn read_utf8_string_lossy(&mut self) -> Result<Cow<'a, str>> {
        let bytes = self.read_utf8_bytes()?;
        Ok(String::from_utf8_lossy(bytes))
    }

    /// Reads a length-prefixed UTF-16 string and returns it as `&[U16<LE>]`.
    ///
    /// This does not copy any data. It reads the prefixed length, locates the contents of the
    /// string, validates that the contents are the right size for UTF-16 (meaning: the length in
    /// bytes is a multiple of 2) and returns the string slice.
    ///
    /// The caller is responsible for converting the returned slice to a different, more usable
    /// form.
    pub fn read_utf16_wchars(&mut self) -> Result<&'a [U16<LE>]> {
        let bytes_len_i32 = self.read_7bit_encoded_i32()?;
        let Ok(bytes_len_usize) = usize::try_from(bytes_len_i32) else {
            return Err(ReaderError::Invalid);
        };

        let bytes = self.read_bytes(bytes_len_usize)?;

        let Ok(wchars) = <[U16<LE>]>::ref_from_bytes(bytes) else {
            return Err(ReaderError::Invalid);
        };

        Ok(wchars)
    }

    /// Reads a length-prefixed UTF-16 string and returns it as `String`.
    ///
    /// The input string is required to be well-formed UTF-16; if it contains illegal UTF-16 code
    /// points or illegal surrogate sequences, then this function will return
    /// `Err(ReaderError::Invalid)`.
    ///
    /// The length in bytes of the string is required to be a multiple of 2. If it is not, then
    /// this function will return `Err(ReaderError::Invalid)`.
    ///
    /// The encoded stream does not contain any information that distinguishes UTF-8 strings and
    /// UTF-16 strings, so applications will need to make sure that they call the correct
    /// `read_utf8_*` or `read_utf16_*` function.
    #[cfg(feature = "std")]
    pub fn read_utf16_string(&mut self) -> Result<String> {
        let wchars = self.read_utf16_wchars()?;
        let wchars_u16: Vec<u16> = wchars.iter().map(|c| c.get()).collect();
        String::from_utf16(&wchars_u16).map_err(|_| ReaderError::Invalid)
    }

    /// Reads a length-prefixed UTF-16 string and returns it as `String`.
    ///
    /// If the input sequence contains illegal UTF-16 code points or illegal surrogate sequences,
    /// then this function will replace the illegal code units with the Unicode replacement
    /// character.
    ///
    /// The length in bytes of the string is required to be a multiple of 2. If it is not, then
    /// this function will return `Err(ReaderError::Invalid)`.
    #[cfg(feature = "std")]
    pub fn read_utf16_string_lossy(&mut self) -> Result<String> {
        let wchars = self.read_utf16_wchars()?;
        let wchars_u16: Vec<u16> = wchars.iter().map(|c| c.get()).collect();
        Ok(String::from_utf16_lossy(&wchars_u16))
    }
}

/// Error type for `BinaryReader`
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ReaderError {
    /// A `read_*` method reached the end of the input data, but requires more data to finish
    /// reading the input.
    ///
    /// If a function returns this error value, then the encoded value may still be well-formed,
    /// if the rest of the data can be read. However, most of the `read_*` functions _do not_
    /// guarantee that they don't advance the read position, even if they return `EndOfData`.
    NeedsMoreData,

    /// The `read_*` request found invalid data in the input. The input is malformed.
    Invalid,
}
