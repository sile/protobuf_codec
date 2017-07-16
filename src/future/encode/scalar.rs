use std::io::Write;
use byteorder::{ByteOrder, LittleEndian};

use Encode;
use future::encode::{EncodeVarint, EncodeLengthDelimited};
use future::write::WriteBytes;
use types;
use wire::types::{Varint, LengthDelimited};

impl<W: Write> Encode<W> for types::Bool {
    type Future = EncodeVarint<W>;
    fn encode(self, writer: W) -> Self::Future {
        Varint(self.0 as u64).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        1
    }
}

impl<W: Write> Encode<W> for types::Uint32 {
    type Future = EncodeVarint<W>;
    fn encode(self, writer: W) -> Self::Future {
        Varint(self.0 as u64).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        Encode::<W>::encoded_size(&Varint(self.0 as u64))
    }
}

impl<W: Write> Encode<W> for types::Uint64 {
    type Future = EncodeVarint<W>;
    fn encode(self, writer: W) -> Self::Future {
        Varint(self.0).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        Encode::<W>::encoded_size(&Varint(self.0))
    }
}

impl<W: Write> Encode<W> for types::Int32 {
    type Future = EncodeVarint<W>;
    fn encode(self, writer: W) -> Self::Future {
        Varint(self.0 as u64).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        Encode::<W>::encoded_size(&Varint(self.0 as u64))
    }
}

impl<W: Write> Encode<W> for types::Int64 {
    type Future = EncodeVarint<W>;
    fn encode(self, writer: W) -> Self::Future {
        Varint(self.0 as u64).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        Encode::<W>::encoded_size(&Varint(self.0 as u64))
    }
}

impl<W: Write> Encode<W> for types::Sint32 {
    type Future = EncodeVarint<W>;
    fn encode(self, writer: W) -> Self::Future {
        let n = self.0 as u32;
        let n = (n << 1) | (n >> 31);
        Varint(n as u64).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        let n = self.0 as u32;
        let n = (n << 1) | (n >> 31);
        Encode::<W>::encoded_size(&Varint(n as u64))
    }
}

impl<W: Write> Encode<W> for types::Sint64 {
    type Future = EncodeVarint<W>;
    fn encode(self, writer: W) -> Self::Future {
        let n = self.0 as u64;
        let n = (n << 1) | (n >> 63);
        Varint(n).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        let n = self.0 as u64;
        let n = (n << 1) | (n >> 63);
        Encode::<W>::encoded_size(&Varint(n))
    }
}

impl<W: Write> Encode<W> for types::Fixed32 {
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(self, writer: W) -> Self::Future {
        let mut bytes = [0; 4];
        LittleEndian::write_u32(&mut bytes, self.0);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(&self) -> u64 {
        4
    }
}

impl<W: Write> Encode<W> for types::Fixed64 {
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(self, writer: W) -> Self::Future {
        let mut bytes = [0; 8];
        LittleEndian::write_u64(&mut bytes, self.0);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(&self) -> u64 {
        8
    }
}

impl<W: Write> Encode<W> for types::Sfixed32 {
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(self, writer: W) -> Self::Future {
        let mut bytes = [0; 4];
        LittleEndian::write_i32(&mut bytes, self.0);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(&self) -> u64 {
        4
    }
}

impl<W: Write> Encode<W> for types::Sfixed64 {
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(self, writer: W) -> Self::Future {
        let mut bytes = [0; 8];
        LittleEndian::write_i64(&mut bytes, self.0);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(&self) -> u64 {
        8
    }
}

impl<W: Write> Encode<W> for types::Float {
    type Future = WriteBytes<W, [u8; 4]>;
    fn encode(self, writer: W) -> Self::Future {
        let mut bytes = [0; 4];
        LittleEndian::write_f32(&mut bytes, self.0);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(&self) -> u64 {
        4
    }
}

impl<W: Write> Encode<W> for types::Double {
    type Future = WriteBytes<W, [u8; 8]>;
    fn encode(self, writer: W) -> Self::Future {
        let mut bytes = [0; 8];
        LittleEndian::write_f64(&mut bytes, self.0);
        WriteBytes::new(writer, bytes)
    }
    fn encoded_size(&self) -> u64 {
        8
    }
}

impl<W: Write> Encode<W> for types::Bytes {
    type Future = EncodeLengthDelimited<W, Vec<u8>>;
    fn encode(self, writer: W) -> Self::Future {
        LengthDelimited(self.0).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        let size = self.0.len() as u64;
        Encode::<W>::encoded_size(&Varint(size)) + size
    }
}

impl<W: Write> Encode<W> for types::Str {
    type Future = EncodeLengthDelimited<W, Vec<u8>>;
    fn encode(self, writer: W) -> Self::Future {
        LengthDelimited(self.0.into_bytes()).encode(writer)
    }
    fn encoded_size(&self) -> u64 {
        let size = self.0.as_bytes().len() as u64;
        Encode::<W>::encoded_size(&Varint(size)) + size
    }
}
