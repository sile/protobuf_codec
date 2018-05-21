//! Encoders and decoders for [Protocol Buffers][protobuf] based on [bytecodec] crate.
//!
//! # Limitation
//!
//! The current version does not support to merge duplicate messages.
//! Alghtough it is required by [The guide][encoding],
//! `protobuf_codec` simply select the last message instance of the same singular field.
//!
//! # Examples
//!
//! An encoder/decoder for `SearchRequest` message defined in the [Language Guide][proto3].
//!
//! ```
//! # extern crate bytecodec;
//! # extern crate protobuf_codec;
//! use bytecodec::EncodeExt;
//! use bytecodec::io::{IoDecodeExt, IoEncodeExt};
//! use protobuf_codec::field::{Fields, FieldDecoder, FieldEncoder, MaybeDefault};
//! use protobuf_codec::field::num::{F1, F2, F3};
//! use protobuf_codec::message::{MessageDecoder, MessageEncoder};
//! use protobuf_codec::scalar::{Int32Decoder, Int32Encoder, StringDecoder, StringEncoder};
//!
//! // syntax = "proto3";
//! //
//! // message SearchRequest {
//! //   string query = 1;
//! //   int32 page_number = 2;
//! //   int32 result_per_page = 3;
//! // }
//! type SearchRequestEncoder = MessageEncoder<
//!     Fields<(
//!         MaybeDefault<FieldEncoder<F1, StringEncoder>>,
//!         MaybeDefault<FieldEncoder<F2, Int32Encoder>>,
//!         MaybeDefault<FieldEncoder<F3, Int32Encoder>>,
//!     )>,
//! >;
//! type SearchRequestDecoder = MessageDecoder<
//!     Fields<(
//!         MaybeDefault<FieldDecoder<F1, StringDecoder>>,
//!         MaybeDefault<FieldDecoder<F2, Int32Decoder>>,
//!         MaybeDefault<FieldDecoder<F3, Int32Decoder>>,
//!     )>,
//! >;
//!
//! # fn main() {
//! let mut buf = Vec::new();
//! let mut encoder = SearchRequestEncoder::with_item(("foo".to_owned(), 3, 10)).unwrap();
//! encoder.encode_all(&mut buf).unwrap();
//! assert_eq!(buf, [10, 3, 102, 111, 111, 16, 3, 24, 10]);
//!
//! let mut decoder = SearchRequestDecoder::default();
//! let message = decoder.decode_exact(&buf[..]).unwrap();
//! assert_eq!(message, ("foo".to_owned(), 3, 10));
//! # }
//! ```
//!
//! # References
//!
//! - [Protocol Buffers: Language Guide (proto2)][proto2]
//! - [Protocol Buffers: Language Guide (proto3)][proto3]
//! - [Protocol Buffers: Encoding][encoding]
//!
//! [bytecodec]: https://github.com/sile/bytecodec
//! [protobuf]: https://developers.google.com/protocol-buffers/docs/overview
//! [proto2]: https://developers.google.com/protocol-buffers/docs/proto
//! [proto3]: https://developers.google.com/protocol-buffers/docs/proto3
//! [encoding]: https://developers.google.com/protocol-buffers/docs/encoding
#![warn(missing_docs)]
#[macro_use]
extern crate bytecodec;
#[macro_use]
extern crate trackable;

#[macro_use]
mod macros;

pub mod field;
pub mod message;
pub mod scalar;
pub mod wire;

mod field_num;
mod fields;
mod oneof;
mod repeated_field;
mod value;

#[cfg(test)]
mod test {
    use bytecodec::EncodeExt;
    use bytecodec::combinator::PreEncode;
    use bytecodec::io::{IoDecodeExt, IoEncodeExt};

    use field::branch::*;
    use field::num::*;
    use field::*;
    use message::*;
    use scalar::*;

    macro_rules! assert_decode {
        ($decoder:ty, $value:expr, $bytes:expr) => {
            let mut decoder: $decoder = Default::default();
            let item = track_try_unwrap!(decoder.decode_exact($bytes.as_ref()));
            assert_eq!(item, $value);
        };
    }

    macro_rules! assert_encode {
        ($encoder:ty, $value:expr, $bytes:expr) => {
            let mut buf = Vec::new();
            let mut encoder: $encoder = track_try_unwrap!(EncodeExt::with_item($value));
            track_try_unwrap!(encoder.encode_all(&mut buf));
            assert_eq!(buf, $bytes);
        };
    }

    fn s(s: &str) -> String {
        s.to_owned()
    }

    // ```proto3
    // // FROM: https://developers.google.com/protocol-buffers/docs/proto3
    //
    // message SearchRequest {
    //   string query = 1;
    //   int32 page_number = 2;
    //   int32 result_per_page = 3;
    // }
    // ```
    type SearchRequestEncoder = MessageEncoder<
        Fields<(
            MaybeDefault<FieldEncoder<F1, StringEncoder>>,
            MaybeDefault<FieldEncoder<F2, Int32Encoder>>,
            MaybeDefault<FieldEncoder<F3, Int32Encoder>>,
        )>,
    >;
    type SearchRequestDecoder = MessageDecoder<
        Fields<(
            MaybeDefault<FieldDecoder<F1, StringDecoder>>,
            MaybeDefault<FieldDecoder<F2, Int32Decoder>>,
            MaybeDefault<FieldDecoder<F3, Int32Decoder>>,
        )>,
    >;

    #[test]
    fn search_request_encoder_works() {
        assert_encode!(
            SearchRequestEncoder,
            (s("foo"), 3, 10),
            [10, 3, 102, 111, 111, 16, 3, 24, 10]
        );

        // The second field is omitted
        assert_encode!(
            SearchRequestEncoder,
            (s("foo"), 0, 10),
            [10, 3, 102, 111, 111, 24, 10]
        );

        // All of the fields are omitted
        assert_encode!(SearchRequestEncoder, (s(""), 0, 0), []);
    }
    #[test]
    fn search_request_decoder_works() {
        assert_decode!(
            SearchRequestDecoder,
            (s("foo"), 3, 10),
            [10, 3, 102, 111, 111, 16, 3, 24, 10]
        );

        // The second field is omitted
        assert_decode!(
            SearchRequestDecoder,
            (s("foo"), 0, 10),
            [10, 3, 102, 111, 111, 24, 10]
        );

        // All of the fields are omitted
        assert_decode!(SearchRequestDecoder, (s(""), 0, 0), []);

        // All of the fields are omitted (only unknown fields are present)
        assert_decode!(
            SearchRequestDecoder,
            (s(""), 0, 0),
            [
                (10 << 3) | 2, // length-delimited
                3,
                102,
                111,
                111,
                (11 << 3) | 0, // varint
                3,
                (12 << 3) | 5, // 32-bit
                10,
                1,
                2,
                3,
                (12 << 3) | 1, // 64-bit
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                8
            ]
        );
    }

    // ```proto3
    // // FROM: https://developers.google.com/protocol-buffers/docs/proto3
    //
    // message SearchResponse {
    //   repeated Result results = 1;
    // }
    //
    // message Result {
    //   string url = 1;
    //   string title = 2;
    //   repeated string snippets = 2;
    // }
    // ```
    type SearchResponseEncoder =
        MessageEncoder<Repeated<Vec<Result>, MessageFieldEncoder<F1, PreEncode<ResultEncoder>>>>;
    type SearchResponseDecoder =
        MessageDecoder<Repeated<Vec<Result>, MessageFieldDecoder<F1, ResultDecoder>>>;

    type Result = (String, String, Vec<String>);
    type ResultEncoder = MessageEncoder<
        Fields<(
            MaybeDefault<FieldEncoder<F1, StringEncoder>>,
            MaybeDefault<FieldEncoder<F2, StringEncoder>>,
            Repeated<Vec<String>, FieldEncoder<F3, StringEncoder>>,
        )>,
    >;
    type ResultDecoder = MessageDecoder<
        Fields<(
            MaybeDefault<FieldDecoder<F1, StringDecoder>>,
            MaybeDefault<FieldDecoder<F2, StringDecoder>>,
            Repeated<Vec<String>, FieldDecoder<F3, StringDecoder>>,
        )>,
    >;

    #[test]
    fn search_response_encoder_works() {
        assert_encode!(
            SearchResponseEncoder,
            vec![(s("foo"), s("111"), vec![s("a"), s("b"), s("c")])],
            [
                10, 19, 10, 3, 102, 111, 111, 18, 3, 49, 49, 49, 26, 1, 97, 26, 1, 98, 26, 1, 99
            ]
        );
    }
    #[test]
    fn search_response_decoder_works() {
        assert_decode!(
            SearchResponseDecoder,
            vec![(s("foo"), s("111"), vec![s("a"), s("b"), s("c")])],
            [
                10, 19, 10, 3, 102, 111, 111, 18, 3, 49, 49, 49, 26, 1, 97, 26, 1, 98, 26, 1, 99
            ]
        );
    }

    // ```proto2
    // // FROM: https://developers.google.com/protocol-buffers/docs/encoding
    //
    // message Test {
    //   repeated int32 d = 4 [packed=true];
    // }
    // ```
    type Test4Encoder = MessageEncoder<PackedFieldEncoder<F4, Vec<i32>, Int32Encoder>>;
    type Test4Decoder = MessageDecoder<PackedFieldDecoder<F4, Vec<i32>, Int32Decoder>>;

    #[test]
    fn test4_encoder_works() {
        assert_encode!(
            Test4Encoder,
            vec![3, 270, 86942],
            [0x22, 0x06, 0x03, 0x8E, 0x02, 0x9E, 0xA7, 0x05]
        );
    }
    #[test]
    fn test4_decoder_works() {
        assert_decode!(
            Test4Decoder,
            vec![3, 270, 86942],
            [0x22, 0x06, 0x03, 0x8E, 0x02, 0x9E, 0xA7, 0x05]
        );
    }

    // ```proto3
    // message MapTest {
    //   map<uint64, bool> entries = 5;
    // }
    // ```
    type MapTestEncoder =
        MessageEncoder<MapFieldEncoder<F5, Vec<(u64, bool)>, Uint64Encoder, BoolEncoder>>;
    type MapTestDecoder =
        MessageDecoder<MapFieldDecoder<F5, Vec<(u64, bool)>, Uint64Decoder, BoolDecoder>>;

    #[test]
    fn map_test_encoder_works() {
        assert_encode!(
            MapTestEncoder,
            vec![(0, true), (11, false), (222, true)],
            [
                42, 4, 8, 0, 16, 1, 42, 4, 8, 11, 16, 0, 42, 5, 8, 222, 1, 16, 1
            ]
        );
    }
    #[test]
    fn map_test_decoder_works() {
        assert_decode!(
            MapTestDecoder,
            vec![(0, true), (11, false), (222, true)],
            [
                42, 4, 8, 0, 16, 1, 42, 4, 8, 11, 16, 0, 42, 5, 8, 222, 1, 16, 1
            ]
        );
    }

    // ```proto3
    // message OneofTest {
    //   oneof test_oneof {
    //     string name = 4;
    //     SearchRequest request = 6;
    //   }
    // }
    // ```
    type OneofTestEncoder = MessageEncoder<
        Optional<
            Oneof<(
                FieldEncoder<F4, StringEncoder>,
                MessageFieldEncoder<F6, SearchRequestEncoder>,
            )>,
        >,
    >;
    type OneofTestDecoder = MessageDecoder<
        Optional<
            Oneof<(
                FieldDecoder<F4, StringDecoder>,
                MessageFieldDecoder<F6, SearchRequestDecoder>,
            )>,
        >,
    >;
    #[test]
    fn oneof_test_encoder_works() {
        assert_encode!(
            OneofTestEncoder,
            Some(Branch2::A(s("foo"))),
            [34, 3, 102, 111, 111]
        );
        assert_encode!(
            OneofTestEncoder,
            Some(Branch2::B(("bar".to_owned(), 3, 10))),
            [50, 9, 10, 3, 98, 97, 114, 16, 3, 24, 10]
        );
        assert_encode!(OneofTestEncoder, None, []);
    }
    #[test]
    fn oneof_test_decoder_works() {
        assert_decode!(
            OneofTestDecoder,
            Some(Branch2::A(s("foo"))),
            [34, 3, 102, 111, 111]
        );
        assert_decode!(
            OneofTestDecoder,
            Some(Branch2::B(("bar".to_owned(), 3, 10))),
            [50, 9, 10, 3, 98, 97, 114, 16, 3, 24, 10]
        );
        assert_decode!(OneofTestDecoder, None, []);

        assert_decode!(
            OneofTestDecoder,
            Some(Branch2::A(s("baz"))),
            [
                34, 3, 102, 111, 111, // A("foo")
                50, 9, 10, 3, 98, 97, 114, 16, 3, 24, 10, // B(("bar", 3, 10))
                34, 3, 98, 97, 122, // A("baz")
            ]
        );
    }
}
