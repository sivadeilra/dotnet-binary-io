# Exchanging data with .NET's `System.IO.BinaryWriter`

This crate provides rudimentary support for exchanging data between Rust code
and C# code, where the data has been encoded using .NET's
`System.IO.BinaryWriter` or needs to be decoded using `System.IO.BinaryReader`.

The encoding scheme for `BinaryWriter` is very simple. Most primitive types (such as integers)
are simply encoded using their fixed-size in-memory byte representation. For multi-byte types,
little-endian byte order is used.

`BinaryWriter` uses a simple encoding for variable-length integers. Refer to the
[.NET documentation](https://learn.microsoft.com/en-us/dotnet/api/system.io.binarywriter.write7bitencodedint?view=net-9.0)
for more information on this encoding.

## Contributing

Contributions are welcome, although I don't expect many for such a small crate.

## Author

* Arlie Davis - `sivadeilra`
