use std::io::Write;

use Encode;
use future::write::WriteBytes;

impl<W: Write> Encode<W> for Vec<u8> {
    type Future = WriteBytes<W, Vec<u8>>;
    fn encode(self, writer: W) -> Self::Future {
        WriteBytes::new(writer, self)
    }
    fn encoded_size(&self) -> u64 {
        self.len() as u64
    }
}
