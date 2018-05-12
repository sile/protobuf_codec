protobuf_codec
==============

[![protobuf_codec](http://meritbadge.herokuapp.com/protobuf_codec)](https://crates.io/crates/protobuf_codec)
[![Documentation](https://docs.rs/protobuf_codec/badge.svg)](https://docs.rs/protobuf_codec)
[![Build Status](https://travis-ci.org/sile/protobuf_codec.svg?branch=master)](https://travis-ci.org/sile/protobuf_codec)
[![Code Coverage](https://codecov.io/gh/sile/protobuf_codec/branch/master/graph/badge.svg)](https://codecov.io/gh/sile/protobuf_codec/branch/master)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust implementation of encoders and decoders for [Protocol Buffers][protobuf] based on [bytecodec] crate.

[Documentation](https://docs.rs/protobuf_codec)

Examples
--------

An encoder/decoder for `SearchRequest` message defined in the [Language Guide][proto3].

```
extern crate bytecodec;
extern crate protobuf_codec;

use bytecodec::EncodeExt;
use bytecodec::io::{IoDecodeExt, IoEncodeExt};
use protobuf_codec::field::{Fields, OptionalFieldDecoder, OptionalFieldEncoder};
use protobuf_codec::field::num::{F1, F2, F3};
use protobuf_codec::message::{MessageDecoder, MessageEncoder};
use protobuf_codec::scalar::{Int32Decoder, Int32Encoder, StringDecoder, StringEncoder};

// syntax = "proto3";
//
// message SearchRequest {
//   string query = 1;
//   int32 page_number = 2;
//   int32 result_per_page = 3;
// }
type SearchRequestEncoder = MessageEncoder<
    Fields<(
        OptionalFieldEncoder<F1, StringEncoder>,
        OptionalFieldEncoder<F2, Int32Encoder>,
        OptionalFieldEncoder<F3, Int32Encoder>,
    )>,
>;
type SearchRequestDecoder = MessageDecoder<
    Fields<(
        OptionalFieldDecoder<F1, StringDecoder>,
        OptionalFieldDecoder<F2, Int32Decoder>,
        OptionalFieldDecoder<F3, Int32Decoder>,
    )>,
>;

fn main() {
    let mut buf = Vec::new();
    let mut encoder = SearchRequestEncoder::with_item(("foo".to_owned(), 3, 10)).unwrap();
    encoder.encode_all(&mut buf).unwrap();
    assert_eq!(buf, [10, 3, 102, 111, 111, 16, 3, 24, 10]);

    let mut decoder = SearchRequestDecoder::default();
    let message = decoder.decode_exact(&buf[..]).unwrap();
    assert_eq!(message, ("foo".to_owned(), 3, 10));
}
```

References
----------

- [Protocol Buffers: Language Guide (proto2)][proto2]
- [Protocol Buffers: Language Guide (proto3)][proto3]
- [Protocol Buffers: Encoding][encoding]

[bytecodec]: https://github.com/sile/bytecodec
[protobuf]: https://developers.google.com/protocol-buffers/docs/overview
[proto2]: https://developers.google.com/protocol-buffers/docs/proto
[proto3]: https://developers.google.com/protocol-buffers/docs/proto3
[encoding]: https://developers.google.com/protocol-buffers/docs/encoding
