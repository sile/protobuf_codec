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
    () => {
        $crate::message::MessageDecoder::new($crate::field::Fields::new(()))
    };
    ($field:tt) => {
        $crate::message::MessageDecoder::new(protobuf_message_field_decoder! $field)
    };
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
        $crate::field::MaybeDefault::new($crate::field::FieldDecoder::new($num, $field))
    };
    ($num:expr, $field:expr, required) => {
        $crate::field::FieldDecoder::new($num, $field)
    };
    ($num:expr, $field:expr, repeated) => {
        $crate::field::Repeated::new($crate::field::FieldDecoder::new($num, $field))
    };
    ($num:expr, $field:expr, $value:ty, repeated) => {
        $crate::field::Repeated::<_, $value>::new($crate::field::FieldDecoder::new($num, $field))
    };
    ($num:expr, $field:expr, packed) => {
        $crate::field::PackedFieldDecoder::new($num, $field)
    };
    ($num:expr, $field:expr, $value:ty, packed) => {
        $crate::field::PackedFieldDecoder::<_, _, $value>::new($num, $field)
    };
    ($num:expr, $field:expr, message) => {
        $crate::field::Optional::new($crate::field::MessageFieldDecoder::new($num, $field))
    };
    ($num:expr, $field:expr, required_message) => {
        $crate::field::MessageFieldDecoder::new($num, $field)
    };
    ($num:expr, $field:expr, repeated_message) => {
        $crate::field::Repeated::new($crate::field::MessageFieldDecoder::new($num, $field))
    };
    ($num:expr, $field:expr, $value:ty, repeated_message) => {
        $crate::field::Repeated::<_, $value>::new($crate::field::MessageFieldDecoder::new($num, $field))
    };
    ($num:expr, $key:expr, $value:expr, map) => {
        $crate::field::MapFieldDecoder::new($num, $key, $value)
    };
    ($num:expr, $key:expr, $value:expr, $map:ty, map) => {
        $crate::field::MapFieldDecoder::<_, _, _, $map>::new($num, $key, $value)
    };
    ($num:expr, $key:expr, $value:expr, map_message) => {
        $crate::field::MapMessageFieldDecoder::new($num, $key, $value)
    };
    ($num:expr, $key:expr, $value:expr, $map:ty, map_message) => {
        $crate::field::MapMessageFieldDecoder::<_, _, _, $map>::new($num, $key, $value)
    };
    (oneof, $($oneof_field:tt),*) => {
        $crate::field::Optional::new(
            $crate::field::Oneof::new(($(protobuf_message_oneof_field_decoder! $oneof_field),*,))
        )
    };
    (required_oneof, $($oneof_field:tt),*) => {
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
        $crate::field::MessageFieldDecoder::new($num, $field)
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
    () => {
        $crate::message::MessageEncoder::new($crate::field::Fields::new(()))
    };
    ($field:tt) => {
        $crate::message::MessageEncoder::new(protobuf_message_field_encoder! $field)
    };
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
        $crate::field::MaybeDefault::new($crate::field::FieldEncoder::new($num, $field))
    };
    ($num:expr, $field:expr, required) => {
        $crate::field::FieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr, repeated) => {
        $crate::field::Repeated::new($crate::field::FieldEncoder::new($num, $field))
    };
    ($num:expr, $field:expr, $value:ty, repeated) => {
        $crate::field::Repeated::<_, $value>::new($crate::field::FieldEncoder::new($num, $field))
    };
    ($num:expr, $field:expr, packed) => {
        $crate::field::PackedFieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr, $value:ty, packed) => {
        $crate::field::PackedFieldEncoder::<_, _, $value>::new($num, $field)
    };
    ($num:expr, $field:expr, message) => {
        $crate::field::Optional::new($crate::field::MessageFieldEncoder::new($num, $field))
    };
    ($num:expr, $field:expr, unsized_message) => {
        $crate::field::Optional::new(
            $crate::field::MessageFieldEncoder::new($num, ::bytecodec::EncodeExt::pre_encode($field))
        )
    };
    ($num:expr, $field:expr, required_message) => {
        $crate::field::MessageFieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr, required_unsized_message) => {
        $crate::field::MessageFieldEncoder::new($num, ::bytecodec::EncodeExt::pre_encode($field))
    };
    ($num:expr, $field:expr, repeated_message) => {
        $crate::field::Repeated::new($crate::field::MessageFieldEncoder::new($num, $field))
    };
    ($num:expr, $field:expr, repeated_unsized_message) => {
        $crate::field::Repeated::new(
            $crate::field::MessageFieldEncoder::new($num, ::bytecodec::EncodeExt::pre_encode($field))
        )
    };
    ($num:expr, $field:expr, $value:ty, repeated_message) => {
        $crate::field::Repeated::<_, $value>::new($crate::field::MessageFieldEncoder::new($num, $field))
    };
    ($num:expr, $field:expr, $value:ty, repeated_unsized_message) => {
        $crate::field::Repeated::<_, $vaule>::new(
            $crate::field::MessageFieldEncoder::new($num, ::bytecodec::EncodeExt::pre_encode($field))
        )
    };
    ($num:expr, $key:expr, $value:expr, map) => {
        $crate::field::MapFieldEncoder::new($num, $key, $value)
    };
    ($num:expr, $key:expr, $value:expr, $map:ty, map) => {
        $crate::field::MapFieldEncoder::<_, _, _, $map>::new($num, $key, $value)
    };
    ($num:expr, $key:expr, $value:expr, map_message) => {
        $crate::field::MapMessageFieldEncoder::new($num, $key, $value)
    };
    ($num:expr, $key:expr, $value:expr, $map:ty, map_message) => {
        $crate::field::MapMessageFieldEncoder::<_, _, _, $map>::new($num, $key, $value)
    };
    ($num:expr, $key:expr, $value:expr, map_unsized_message) => {
        $crate::field::MapMessageFieldEncoder::new($num, $key, ::bytecodec::EncodeExt::pre_encode($value))
    };
    ($num:expr, $key:expr, $value:expr, $map:ty, map_unsized_message) => {
        $crate::field::MapMessageFieldEncoder::<_, _, _, $map>::new(
            $num, $key, ::bytecodec::EncodeExt::pre_encode($value)
        )
    };
    (oneof, $($oneof_field:tt),*) => {
        $crate::field::Optional::new(
            $crate::field::Oneof::new(($(protobuf_message_oneof_field_encoder! $oneof_field),*,))
        )
    };
    (required_oneof, $($oneof_field:tt),*) => {
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
        $crate::field::MessageFieldEncoder::new($num, $field)
    };
    ($num:expr, $field:expr,unsized_message) => {
        $crate::field::MessageFieldEncoder::new($num, ::bytecodec::EncodeExt::pre_encode($field))
    };
}

#[cfg(test)]
mod tests {
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
        let v2 = (v1, vec![4], Some(Branch2::A(5)));

        let mut decoder = m2;
        assert_eq!(
            decoder
                .decode_from_bytes(&[10, 8, 10, 4, 8, 1, 16, 2, 16, 3, 18, 1, 4, 24, 5][..])
                .unwrap(),
            v2
        );
    }

    #[test]
    fn single_decoder_works() {
        let mut decoder = protobuf_message_decoder![(F1, Int32Decoder::new())];
        assert_eq!(decoder.decode_from_bytes(&[8, 10]).unwrap(), 10);
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
        let mut m2 = protobuf_message_encoder![
            (F1, m1, required_unsized_message),
            (F2, Int32Encoder::new(), Vec<i32>, packed),
            (oneof, (F3, Int32Encoder::new()), (F4, Uint64Encoder::new()))
        ];

        let v0 = (1, 2);
        let v1 = (vec![v0], vec![3]);
        let v2 = (v1, vec![4], Some(Branch2::A(5)));

        assert_eq!(
            m2.encode_into_bytes(v2).unwrap(),
            [10, 8, 10, 4, 8, 1, 16, 2, 16, 3, 18, 1, 4, 24, 5]
        );
    }

    #[test]
    fn single_encoder_works() {
        let mut encoder = protobuf_message_encoder![(F1, Int32Encoder::new())];
        assert_eq!(encoder.encode_into_bytes(10).unwrap(), [8, 10]);
    }

    #[test]
    fn repeated_encoder_works() {
        let mut encoder = protobuf_message_encoder!(
            (F1, StringEncoder::new(), repeated),
            (F2, Uint32Encoder::new())
        );
        assert_eq!(
            encoder.encode_into_bytes((vec!["foo"], 0)).unwrap(),
            [10, 3, 102, 111, 111]
        )
    }
}
