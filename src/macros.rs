/// Macro for creating an instance of a message decoder.
///
/// # Examples
///
/// ```
/// # extern crate bytecodec;
/// # #[macro_use]
/// # extern crate protobuf_codec;
/// use bytecodec::DecodeExt;
/// use protobuf_codec::field::num::{F1, F2};
/// use protobuf_codec::scalar::Int32Decoder;
///
/// // syntax = "proto3";
/// //
/// // message Pair {
/// //   int32 aaa = 1;
/// //   int32 bbb = 2;
/// // }
///
/// # fn main() {
/// let mut decoder = protobuf_message_decoder![
///     (F1, Int32Decoder::new()),
///     (F2, Int32Decoder::new(), required)
/// ];
/// assert_eq!(decoder.decode_from_bytes(&[8, 1, 16, 2][..]).unwrap(), (1, 2));
/// # }
/// ```
#[macro_export]
macro_rules! protobuf_message_decoder {
    ($($field:tt),*) => {
        $crate::message::MessageDecoder::new(
            $crate::field::Fields::new(($(protobuf_message_field_decoder! $field),*,))
        )
    };
}

/// Auxiliary macro used for expanding `$field` argument of `protobuf_message_decoder!`.
#[macro_export]
macro_rules! protobuf_message_field_decoder {
    ($num:expr, $field:expr) => {
        $crate::field::OptionalFieldDecoder::new($num, $field)
    };
    ($num:expr, $field:expr, required) => {
        $crate::field::FieldDecoder::new($num, $field)
    };
    ($num:expr, $field:expr, repeated) => {
        $crate::field::RepeatedFieldDecoder::new($num, $field)
    };
    ($num:expr, $field:expr, $value:ty, repeated) => {
        $crate::field::RepeatedFieldDecoder::<_, $value, _>::new($num, $field)
    };
    ($num:expr, $field:expr, packed) => {
        $crate::field::PackedRepeatedFieldDecoder::new($num, $field)
    };
    ($num:expr, $field:expr, $value:ty, packed) => {
        $crate::field::PackedRepeatedFieldDecoder::<_, $value, _>::new($num, $field)
    };
    ($num:expr, $field:expr, message) => {
        $crate::field::OptionalMessageFieldDecoder::new($num, $crate::message::EmbeddedMessageDecoder::new($field))
    };
    ($num:expr, $field:expr, required_message) => {
        $crate::field::MessageFieldDecoder::new(
            $num,
            $crate::message::EmbeddedMessageDecoder::new($field)
        )
    };
    ($num:expr, $field:expr, repeated_message) => {
        $crate::field::RepeatedMessageFieldDecoder::new(
            $num,
            $crate::message::EmbeddedMessageDecoder::new($field)
        )
    };
    ($num:expr, $field:expr, $value:ty, repeated_message) => {
        $crate::field::RepeatedMessageFieldDecoder::<_, $value, _>::new(
            $num,
            $crate::message::EmbeddedMessageDecoder::new($field)
        )
    };
    (oneof, $($oneof_field:tt),*) => {
        $crate::field::Oneof::new(($(protobuf_message_oneof_field_decoder! $oneof_field),*,))
    };
}

/// Auxiliary macro used for expanding `$oneof_field` argument of `protobuf_message_field_decoder!`.
#[macro_export]
macro_rules! protobuf_message_oneof_field_decoder {
    ($num:expr, $field:expr) => {
        $crate::field::FieldDecoder::new($num, $field)
    };
    ($num:expr, $field:expr,message) => {
        $crate::field::MessageFieldDecoder::new(
            $num,
            $crate::message::EmbeddedMessageDecoder::new($field),
        )
    };
    ($num:expr, $field:expr,unsized_message) => {
        $crate::field::MessageFieldDecoder::new(
            $num,
            $crate::message::EmbeddedMessageDecoder::new(::bytecodec::EncodeExt::pre_encode(
                $field,
            )),
        )
    };
}

/// Macro for creating an instance of a message encoder.
///
/// # Examples
///
/// ```
/// # extern crate bytecodec;
/// # #[macro_use]
/// # extern crate protobuf_codec;
/// use bytecodec::EncodeExt;
/// use protobuf_codec::field::num::{F1, F2};
/// use protobuf_codec::scalar::Int32Encoder;
///
/// // syntax = "proto3";
/// //
/// // message Pair {
/// //   int32 aaa = 1;
/// //   int32 bbb = 2;
/// // }
///
/// # fn main() {
/// let mut encoder = protobuf_message_encoder![
///     (F1, Int32Encoder::new()),
///     (F2, Int32Encoder::new(), required)
/// ];
/// assert_eq!(encoder.encode_into_bytes((1,2)).unwrap(), [8, 1, 16, 2]);
/// # }
/// ```
#[macro_export]
macro_rules! protobuf_message_encoder {
    ($($field:tt),*) => {
        $crate::message::MessageEncoder::new(
            $crate::field::Fields::new(($(protobuf_message_field_encoder! $field),*,))
        )
    };
}

/// Auxiliary macro used for expanding `$field` argument of `protobuf_message_encoder!`.
#[macro_export]
macro_rules! protobuf_message_field_encoder {
    ($num:expr, $field:expr) => {
        $crate::field::OptionalFieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr, required) => {
        $crate::field::FieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr, repeated) => {
        $crate::field::RepeatedFieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr, $value:ty, repeated) => {
        $crate::field::RepeatedFieldEncoder::<_, $value, _>::new($num, $field)
    };
    ($num:expr, $field:expr, packed) => {
        $crate::field::PackedRepeatedFieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr, $value:ty, packed) => {
        $crate::field::PackedRepeatedFieldEncoder::<_, $value, _>::new($num, $field)
    };
    ($num:expr, $field:expr, message) => {
        $crate::field::OptionalMessageFieldEncoder::new($num, $crate::message::EmbeddedMessageEncoder::new($field))
    };
    ($num:expr, $field:expr, unsized_message) => {
        $crate::field::OptionalMessageFieldEncoder::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new(::bytecodec::EncodeExt::pre_encode($field))
        )
    };
    ($num:expr, $field:expr, required_message) => {
        $crate::field::MessageFieldEncoder::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new($field)
        )
    };
    ($num:expr, $field:expr, required_unsized_message) => {
        $crate::field::MessageFieldEncoder::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new(::bytecodec::EncodeExt::pre_encode($field))
        )
    };
    ($num:expr, $field:expr, repeated_message) => {
        $crate::field::RepeatedMessageFieldEncoder::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new($field)
        )
    };
    ($num:expr, $field:expr, repeated_unsized_message) => {
        $crate::field::RepeatedMessageFieldEncoder::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new(::bytecodec::EncodeExt::pre_encode($field))
        )
    };
    ($num:expr, $field:expr, $value:ty, repeated_message) => {
        $crate::field::RepeatedMessageFieldEncoder::<_, $value, _>::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new($field)
        )
    };
    ($num:expr, $field:expr, $value:ty, repeated_unsized_message) => {
        $crate::field::RepeatedMessageFieldEncoder::<_, $value, _>::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new(::bytecodec::EncodeExt::pre_encode($field))
        )
    };
    (oneof, $($oneof_field:tt),*) => {
        $crate::field::Oneof::new(($(protobuf_message_oneof_field_encoder! $oneof_field),*,))
    };
}

/// Auxiliary macro used for expanding `$oneof_field` argument of `protobuf_message_field_encoder!`.
#[macro_export]
macro_rules! protobuf_message_oneof_field_encoder {
    ($num:expr, $field:expr) => {
        $crate::field::FieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr,message) => {
        $crate::field::MessageFieldEncoder::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new($field),
        )
    };
    ($num:expr, $field:expr,unsized_message) => {
        $crate::field::MessageFieldEncoder::new(
            $num,
            $crate::message::EmbeddedMessageEncoder::new(::bytecodec::EncodeExt::pre_encode(
                $field,
            )),
        )
    };
}

#[cfg(test)]
mod test {
    use bytecodec::{DecodeExt, EncodeExt};

    use field::branch::*;
    use field::num::*;
    use scalar::*;

    #[test]
    fn decoder_macro_works() {
        let m0 = protobuf_message_decoder![
            (F1, Int32Decoder::new()),
            (F2, Int32Decoder::new(), required)
        ];
        let m1 = protobuf_message_decoder![
            (F1, m0, Vec<(i32, i32)>, repeated_message),
            (F2, Int32Decoder::new(), Vec<i32>, repeated)
        ];
        let m2 = protobuf_message_decoder![
            (F1, m1, required_message),
            (F2, Int32Decoder::new(), Vec<i32>, packed),
            (oneof, (F3, Int32Decoder::new()), (F4, Uint64Decoder::new()))
        ];

        let v0 = (1, 2);
        let v1 = (vec![v0], vec![3]);
        let v2 = (v1, vec![4], Branch2::A(5));

        let mut decoder = m2;
        assert_eq!(
            decoder
                .decode_from_bytes(&[10, 8, 10, 4, 8, 1, 16, 2, 16, 3, 18, 1, 4, 24, 5][..])
                .unwrap(),
            v2
        );
    }

    #[test]
    fn encoder_macro_works() {
        let m0 = protobuf_message_encoder![
            (F1, Int32Encoder::new()),
            (F2, Int32Encoder::new(), required)
        ];
        let m1 = protobuf_message_encoder![
            (F1, m0, Vec<(i32, i32)>, repeated_message),
            (F2, Int32Encoder::new(), Vec<i32>, repeated)
        ];
        let m2 = protobuf_message_encoder![
            (F1, m1, required_unsized_message),
            (F2, Int32Encoder::new(), Vec<i32>, packed),
            (oneof, (F3, Int32Encoder::new()), (F4, Uint64Encoder::new()))
        ];

        let v0 = (1, 2);
        let v1 = (vec![v0], vec![3]);
        let v2 = (v1, vec![4], Branch2::A(5));

        let mut encoder = m2.pre_encode();
        assert_eq!(
            encoder.encode_into_bytes(v2).unwrap(),
            [10, 8, 10, 4, 8, 1, 16, 2, 16, 3, 18, 1, 4, 24, 5]
        );
    }

    #[test]
    fn repeated_encoder_works() {
        let mut encoder = protobuf_message_encoder!(
            (F1, StringEncoder::new(), repeated),
            (F2, Uint32Encoder::new())
        ).pre_encode();
        assert_eq!(
            encoder.encode_into_bytes((vec!["foo"], 0)).unwrap(),
            [10, 3, 102, 111, 111]
        )
    }
}
