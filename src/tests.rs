use crate::*;
use pretty_hex::PrettyHex;

#[test]
fn basic_u8() {
    let mut r = BinaryReader::new(&[42, 43, 44]);
    let b = r.read_u8().unwrap();
    assert_eq!(b, 42);
    assert_eq!(r.data, &[43, 44]);
}

/// Not sure why you would ever test a zero-length read, but ok.
#[test]
fn read_cbytes_zero_len() {
    let mut r = BinaryReader::new(&[0x33, 0x44]);
    let _empty: [u8; 0] = r.read_cbytes().unwrap();
    assert_eq!(r.data, [0x33, 0x44]);
}

#[test]
fn read_cbytes_not_enough() {
    let mut r = BinaryReader::new(&[0x33, 0x44, 0x55]);
    assert_eq!(r.read_cbytes::<5>(), Err(ReaderError::NeedsMoreData));
}

#[test]
fn read_cbytes_some() {
    let mut r = BinaryReader::new(&[0x33, 0x44, 0x55]);
    assert_eq!(r.read_cbytes(), Ok([0x33, 0x44]));
    assert_eq!(r.data, [0x55]);
}

#[test]
fn read_u8() {
    let mut r = BinaryReader::new(&[0x33, 0x44, 0x55]);
    assert_eq!(r.read_u8(), Ok(0x33));
    assert_eq!(r.data, [0x44, 0x55]);
}

#[test]
fn basic_u16() {
    let mut r = BinaryReader::new(&[]);
    assert_eq!(r.read_u16(), Err(ReaderError::NeedsMoreData));

    let mut r = BinaryReader::new(&[0xaa, 0x55, 0x33, 0x44]);
    assert_eq!(r.read_u16(), Ok(0x55aa));
    assert_eq!(r.data, &[0x33, 0x44]);
}

#[test]
fn str_utf8() {
    let mut w = BinaryWriter::new();
    w.write_utf8_str("Hello!").unwrap();
    w.write_u16(0xaa55);
    assert_eq!(w.out, [6, b'H', b'e', b'l', b'l', b'o', b'!', 0x55, 0xaa]);

    let mut r = BinaryReader::new(&w.out);
    assert_eq!(r.read_utf8_bytes(), Ok(b"Hello!".as_slice()));
    assert_eq!(r.data, [0x55, 0xaa]);
}

#[test]
fn str_utf16() {
    let mut w = BinaryWriter::new();
    w.write_utf16_encode("Hello!");
}

#[test]
fn mixed() {
    let mut w = BinaryWriter::new();
    w.write_u8(42);
    w.write_u16(0x0102);
    w.write_utf8_str("Hello, world!").unwrap();
    w.write_i32(-33);

    println!("{}", w.out.hex_dump());

    let mut r = BinaryReader::new(&w.out);
    assert_eq!(r.read_u8(), Ok(42));
    assert_eq!(r.read_u16(), Ok(0x0102));
    assert_eq!(r.read_utf8_bytes(), Ok(b"Hello, world!".as_slice()));
    assert_eq!(r.read_i32(), Ok(-33));
}

#[test]
fn int7_i32() {
    let cases: &[(i32, &[u8])] = &[
        (0 /* 0x00000000 */, &[0x00]),
        (1 /* 0x00000001 */, &[0x01]),
        (-1 /* 0xffffffff */, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
        (127 /* 0x0000007f */, &[0x7f]),
        (128 /* 0x00000080 */, &[0x80, 0x01]),
        (255 /* 0x000000ff */, &[0xff, 0x01]),
        (256 /* 0x00000100 */, &[0x80, 0x02]),
        (
            -12345, /* 0xffffcfc7 */
            &[0xc7, 0x9f, 0xff, 0xff, 0x0f],
        ),
        (12345 /* 0x00003039 */, &[0xb9, 0x60]),
        (
            2147483647, /* 0x7fffffff */
            &[0xff, 0xff, 0xff, 0xff, 0x07],
        ),
        (
            -2147483648, /* 0x80000000 */
            &[0x80, 0x80, 0x80, 0x80, 0x08],
        ),
    ];

    // Check encoding
    for &(x, bytes) in cases.iter() {
        let mut w = BinaryWriter::new();
        w.write_7bit_encoded_i32(x);
        assert_eq!(w.out, bytes, "x = {x} (0x{x:x})");
    }

    // Check decoding
    for &(expected_x, bytes) in cases.iter() {
        let mut r = BinaryReader::new(bytes);
        let decoded_x = r.read_7bit_encoded_i32().unwrap();
        assert_eq!(decoded_x, expected_x, "x = {expected_x} (0x{expected_x:x})");
    }
}

#[test]
fn int7_i64() {
    let cases: &[(i64, &[u8])] = &[
        (0 /* 0x00000000 */, &[0x00]),
        (1 /* 0x00000001 */, &[0x01]),
        (
            -1, /* 0xffffffffffffffff */
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (127 /* 0x0000007f */, &[0x7f]),
        (128 /* 0x00000080 */, &[0x80, 0x01]),
        (255 /* 0x000000ff */, &[0xff, 0x01]),
        (256 /* 0x00000100 */, &[0x80, 0x02]),
        (
            -12345, /* 0xffffffffffffcfc7 */
            &[0xc7, 0x9f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (12345 /* 0x00003039 */, &[0xb9, 0x60]),
        (
            9223372036854775807, /* 0x7fffffffffffffff */
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        ),
        (
            -9223372036854775808, /* 0x8000000000000000 */
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ),
    ];

    // Check encoding
    for &(x, bytes) in cases.iter() {
        let mut w = BinaryWriter::new();
        w.write_7bit_encoded_i64(x);
        assert_eq!(w.out, bytes, "x = {x} (0x{x:x})");
    }

    // Check decoding
    for &(expected_x, bytes) in cases.iter() {
        let mut r = BinaryReader::new(bytes);
        let decoded_x = r.read_7bit_encoded_i64().unwrap();
        assert_eq!(decoded_x, expected_x, "x = {expected_x} (0x{expected_x:x})");
    }
}
