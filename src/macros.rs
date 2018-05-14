/// Macro for defining the type of a message encoder.
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
/// type PairEncoder = protobuf_message_encoder![
///     (F1, Int32Encoder),
///     (F2, Int32Encoder, required)
/// ];
///
/// # fn main() {
/// let mut encoder = PairEncoder::default();
/// assert_eq!(encoder.encode_into_bytes((1,2)).unwrap(), [8, 1, 16, 2]);
/// # }
/// ```
#[macro_export]
macro_rules! protobuf_message_encoder {
    ($($field:tt),*) => {
        $crate::message::MessageEncoder<
            $crate::field::Fields<($(protobuf_message_field_encoder! $field),*,)>
         >
    };
}

/// Auxiliary macro used for expanding `$field` argument of `protobuf_message_encoder!`.
#[macro_export]
macro_rules! protobuf_message_field_encoder {
    ($num:ty, $field:ty) => {
        $crate::field::OptionalFieldEncoder<$num, $field>
    };
    ($num:ty, $field:ty, required) => {
        $crate::field::FieldEncoder<$num, $field>
    };
    ($num:ty, $field:ty, repeated) => {
        $crate::field::RepeatedFieldEncoder<$num, _, $field>
    };
    ($num:ty, $field:ty, $value:ty, repeated) => {
        $crate::field::RepeatedFieldEncoder<$num, $value, $field>
    };
    ($num:ty, $field:ty, packed) => {
        $crate::field::PackedRepeatedFieldEncoder<$num, _, $field>
    };
    ($num:ty, $field:ty, $value:ty, packed) => {
        $crate::field::PackedRepeatedFieldEncoder<$num, $value, $field>
    };
    ($num:ty, $field:ty, message) => {
        $crate::field::OptionalMessageFieldEncoder<$num, $field>
    };
    ($num:ty, $field:ty, unsized_message) => {
        $crate::field::OptionalMessageFieldEncoder<$num, ::bytecodec::combinator::PreEncode<$field>>
    };
    ($num:ty, $field:ty, required_message) => {
        $crate::field::MessageFieldEncoder<$num, $field>
    };
    ($num:ty, $field:ty, required_unsized_message) => {
        $crate::field::MessageFieldEncoder<$num, ::bytecodec::combinator::PreEncode<$field>>
    };
    ($num:ty, $field:ty, repeated_message) => {
        $crate::field::RepeatedMessageFieldEncoder<$num, _, $field>
    };
    ($num:ty, $field:ty, repeated_unsized_message) => {
        $crate::field::RepeatedMessageFieldEncoder<$num, _, ::bytecodec::combinator::PreEncode<$field>>
    };
    ($num:ty, $field:ty, $value:ty, repeated_message) => {
        $crate::field::RepeatedMessageFieldEncoder<$num, $value, $field>
    };
    ($num:ty, $field:ty, $value:ty, repeated_unsized_message) => {
        $crate::field::RepeatedMessageFieldEncoder<$num, $value, ::bytecodec::combinator::PreEncode<$field>>
    };
    (oneof, $($oneof_field:tt),* ) => {
        $crate::field::Oneof<($(protobuf_message_oneof_field_encoder! $oneof_field),*,)>
    };
}

/// Auxiliary macro used for expanding `$oneof_field` argument of `protobuf_message_field_encoder!`.
#[macro_export]
macro_rules! protobuf_message_oneof_field_encoder {
    ($num:ty, $field:ty) => {
        $crate::field::FieldEncoder<$num, $field>
    };
    ($num:ty, $field:ty, message) => {
        $crate::field::MessageFieldEncoder<$num, $field>
    };
    ($num:ty, $field:ty, unsized_message) => {
        $crate::field::MessageFieldEncoder<$num, ::bytecodec::combinator::PreEncode<$field>>
    };
}

#[cfg(test)]
mod test {
    use bytecodec::EncodeExt;

    use field::branch::*;
    use field::num::*;
    use scalar::*;

    #[test]
    fn encoder_macro_works() {
        type M0 = protobuf_message_encoder![(F1, Int32Encoder), (F2, Int32Encoder, required)];
        type M1 = protobuf_message_encoder![
            (F1, M0, Vec<(i32, i32)>, repeated_message),
            (F2, Int32Encoder, Vec<i32>, repeated)
        ];
        type M2 = protobuf_message_encoder![
            (F1, M1, required_unsized_message),
            (F2, Int32Encoder, Vec<i32>, packed),
            (oneof, (F3, Int32Encoder), (F4, M0, message))
        ];

        let v0 = (1, 2);
        let v1 = (vec![v0], vec![3]);
        let v2 = (v1, vec![4], Branch2::A(5));

        let mut encoder = M2::default().pre_encode();
        assert_eq!(
            encoder.encode_into_bytes(v2).unwrap(),
            [10, 8, 10, 4, 8, 1, 16, 2, 16, 3, 18, 1, 4, 24, 5]
        );
    }
}
