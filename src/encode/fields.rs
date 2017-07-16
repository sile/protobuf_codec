use std::io::Write;
use futures::{Future, Poll, Async};
use Error;
use fields;
use traits::{Tag, Type};
use util_futures::{Phase2, WithState};
use wire::types::Varint;
use super::Encode;
use super::futures::EncodeTagAndWireType;

#[derive(Debug)]
pub struct EncodeField<W, F>
where
    W: Write,
    F: Encode<W>,
{
    phase: Phase2<WithState<EncodeTagAndWireType<W>, F::Value>, F::Future>,
}
impl<W: Write, F: Encode<W>> Future for EncodeField<W, F> {
    type Item = W;
    type Error = Error<W>;
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        while let Async::Ready(phase) = track!(self.phase.poll())? {
            let next = match phase {
                Phase2::A((w, v)) => Phase2::B(F::encode(w, v)),
                Phase2::B(w) => return Ok(Async::Ready(w)),
            };
            self.phase = next;
        }
        Ok(Async::NotReady)
    }
}
impl<W: Write, T: Tag, F: Type + Encode<W>> Encode<W> for fields::Field<T, F> {
    type Future = EncodeField<W, F>;
    fn encode(writer: W, value: Self::Value) -> Self::Future {
        let future = EncodeTagAndWireType::new(writer, T::number(), F::wire_type());
        let phase = Phase2::A(WithState::new(future, value));
        EncodeField { phase }
    }
    fn encoded_size(value: &Self::Value) -> u64 {
        let key_size = <Varint as Encode<W>>::encoded_size(&((T::number() as u64) << 3));
        let value_size = F::encoded_size(value);
        key_size + value_size
    }
}

// pub struct EncodeRepeated<W, T>
// where
//     W: Write,
//     T: Encode<W>,
// {
//     tag: Tag,
//     wire_type: WireType,
//     values: Vec<T::Value>,
//     future: Either<EncodeField<W, T>, Finished<W, EncodeError<W>>>,
// }
// impl<W: Write, T: Encode<W>> EncodeRepeated<W, T> {
//     fn new(tag: Tag, wire_type: WireType, mut values: Vec<T::Value>, writer: W) -> Self {
//         values.reverse();
//         let future = if let Some(v) = values.pop() {
//             Either::A(EncodeField::new(tag, wire_type, v, writer))
//         } else {
//             Either::B(futures::finished(writer))
//         };
//         EncodeRepeated {
//             tag,
//             wire_type,
//             values,
//             future,
//         }
//     }
// }
// impl<W: Write, T: Encode<W>> Future for EncodeRepeated<W, T> {
//     type Item = W;
//     type Error = EncodeError<W>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         while let Async::Ready(w) = track!(self.future.poll())? {
//             if let Some(v) = self.values.pop() {
//                 let future = Either::A(EncodeField::new(self.tag, self.wire_type, v, w));
//                 self.future = future;
//             } else {
//                 return Ok(Async::Ready(w));
//             }
//         }
//         Ok(Async::NotReady)
//     }
// }
// impl<W: Write, T> Encode<W> for fields::Repeated<T>
// where
//     T: Type + Encode<W>,
// {
//     type Value = (Tag, Vec<<T as Encode<W>>::Value>);
//     type Future = EncodeRepeated<W, T>;
//     fn encode(value: Self::Value, writer: W) -> Self::Future {
//         EncodeRepeated::new(value.0, T::wire_type(), value.1, writer)
//     }
//     fn encoded_size(value: &Self::Value) -> u64 {
//         let key_size = <Varint as Encode<W>>::encoded_size(&(((value.0).0 << 3) as u64));
//         value.1.iter().map(|v| key_size + T::encoded_size(v)).sum()
//     }
// }

// #[derive(Debug)]
// pub struct EncodePackedRepeated<W, T>
// where
//     W: Write,
//     T: Encode<W>,
// {
//     values: Vec<T::Value>,
//     phase: Phase3<EncodeTagAndWireType<W>, T::Future, Finished<W, EncodeError<W>>>,
// }
// impl<W: Write, T: Encode<W>> EncodePackedRepeated<W, T> {
//     fn new(tag: Tag, wire_type: WireType, mut values: Vec<T::Value>, writer: W) -> Self {
//         values.reverse();
//         let phase = if values.is_empty() {
//             Phase3::C(futures::finished(writer))
//         } else {
//             Phase3::A(EncodeTagAndWireType::new(tag, wire_type, writer))
//         };
//         EncodePackedRepeated { values, phase }
//     }
// }
// impl<W: Write, T: Encode<W>> Future for EncodePackedRepeated<W, T> {
//     type Item = W;
//     type Error = EncodeError<W>;
//     fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
//         while let Async::Ready(phase) = track!(self.phase.poll())? {
//             let w = match phase {
//                 Phase3::A(w) => w,
//                 Phase3::B(w) => w,
//                 Phase3::C(w) => w,
//             };
//             if let Some(v) = self.values.pop() {
//                 self.phase = Phase3::B(T::encode(v, w));
//             } else {
//                 return Ok(Async::Ready(w));
//             }
//         }
//         Ok(Async::NotReady)
//     }
// }
// impl<W: Write, T> Encode<W> for fields::PackedRepeated<T>
// where
//     T: Type + Encode<W>,
// {
//     type Value = (Tag, Vec<<T as Encode<W>>::Value>);
//     type Future = EncodePackedRepeated<W, T>;
//     fn encode(value: Self::Value, writer: W) -> Self::Future {
//         EncodePackedRepeated::new(value.0, T::wire_type(), value.1, writer)
//     }
//     fn encoded_size(value: &Self::Value) -> u64 {
//         let key_size = <Varint as Encode<W>>::encoded_size(&(((value.0).0 << 3) as u64));
//         key_size + value.1.iter().map(T::encoded_size).sum::<u64>()
//     }
// }

// impl<W: Write, A, B> Encode<W> for fields::Oneof<(A, B)>
// where
//     A: Encode<W>,
//     B: Encode<W>,
// {
//     type Value = Variant2<A::Value, B::Value>;
//     type Future = EncodeVariant2<W, A, B>;
//     fn encode(value: Self::Value, writer: W) -> Self::Future {
//         EncodeVariant2::new(value, writer)
//     }
//     fn encoded_size(value: &Self::Value) -> u64 {
//         match *value {
//             Variant2::A(ref v) => A::encoded_size(v),
//             Variant2::B(ref v) => B::encoded_size(v),
//             Variant2::None => 0,
//         }
//     }
// }
