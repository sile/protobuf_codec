use std::io::{self, Read};

use decode::{Decode, FieldDecode};
use decode::futures;

#[derive(Debug, Clone, Copy)]
pub struct Bool;
impl<R: Read> Decode<R> for Bool {
    type Value = bool;
    type Future = futures::DecodeBool<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeBool::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Uint32;
impl<R: Read> Decode<R> for Uint32 {
    type Value = u32;
    type Future = futures::DecodeUint32<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeUint32::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Uint64;
impl<R: Read> Decode<R> for Uint64 {
    type Value = u64;
    type Future = futures::DecodeUint64<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeUint64::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Int32;
impl<R: Read> Decode<R> for Int32 {
    type Value = i32;
    type Future = futures::DecodeInt32<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeInt32::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Int64;
impl<R: Read> Decode<R> for Int64 {
    type Value = i64;
    type Future = futures::DecodeInt64<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeInt64::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sint32;
impl<R: Read> Decode<R> for Sint32 {
    type Value = i32;
    type Future = futures::DecodeSint32<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeSint32::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sint64;
impl<R: Read> Decode<R> for Sint64 {
    type Value = i64;
    type Future = futures::DecodeSint64<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeSint64::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fixed32;
impl<R: Read> Decode<R> for Fixed32 {
    type Value = u32;
    type Future = futures::DecodeFixed32<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeFixed32::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sfixed32;
impl<R: Read> Decode<R> for Sfixed32 {
    type Value = i32;
    type Future = futures::DecodeSfixed32<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeSfixed32::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Float;
impl<R: Read> Decode<R> for Float {
    type Value = f32;
    type Future = futures::DecodeFloat<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeFloat::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Fixed64;
impl<R: Read> Decode<R> for Fixed64 {
    type Value = u64;
    type Future = futures::DecodeFixed64<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeFixed64::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sfixed64;
impl<R: Read> Decode<R> for Sfixed64 {
    type Value = i64;
    type Future = futures::DecodeSfixed64<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeSfixed64::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Double;
impl<R: Read> Decode<R> for Double {
    type Value = f64;
    type Future = futures::DecodeDouble<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeDouble::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Bytes;
impl<R: Read> Decode<R> for Bytes {
    type Value = Vec<u8>;
    type Future = futures::DecodeBytes<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeBytes::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Utf8;
impl<R: Read> Decode<R> for Utf8 {
    type Value = String;
    type Future = futures::DecodeUtf8<R>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodeUtf8::new(reader)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Packed<T>(pub T);
impl<R: Read, T: Clone + Decode<io::Take<R>>> Decode<R> for Packed<T> {
    type Value = Vec<T::Value>;
    type Future = futures::DecodePacked<R, T>;
    fn decode(&self, reader: R) -> Self::Future {
        futures::DecodePacked::new(self.0.clone(), reader)
    }
}

// #[derive(Debug, Clone, Copy)]
// pub struct Message2<F0, F1> {
//     pub name: &'static str,
//     pub field0: F0,
//     pub field1: F1,
// }
// impl<R: Read> Decode<R> for Message2<F0, F1>
// where
//     F0: FieldDecode<R>,
//     F1: FieldDecode<R>,
// {
//     type Value = (F0::Value, F1::Value);
//     type Future = futures::DecodeMessage2<R, F0, F1>;
//     fn decode(&self, reader: R) -> Self::Future {
//         futures::DecodeMessage2::new(self, reader)
//     }
// }

#[cfg(test)]
mod test {
    use futures::Future;

    use decode::Decode;
    use super::*;

    #[test]
    fn decode_bool() {
        let input = [0b0000_0001];
        let (_, b) = track_try_unwrap!(Bool.decode(&input[..]).wait());
        assert_eq!(b, true);

        let input = [0b0000_0000];
        let (_, b) = track_try_unwrap!(Bool.decode(&input[..]).wait());
        assert_eq!(b, false);
    }
}
