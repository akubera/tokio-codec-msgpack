//!
//! Tokio & MsgPack
//!

pub use bytes::BytesMut;
use rmpv;

use tokio_codec::length_delimited::{Builder as BuilderLDC, LengthDelimitedCodec};
use tokio_codec::{Decoder, Encoder};

/// Turn tokio stream into MsgPack messages
///
/// Messages are length-prefixed with u32 big-endian integers of the : `u32
///
pub struct MsgPackCodec {
  // building params
  // params: Builder,

  // the underlying byte-codec
  inner: LengthDelimitedCodec,
}

impl MsgPackCodec {
  pub fn builder() -> Builder
  {
    Builder::default()
  }

  pub fn max_frame_length(&self) -> usize
  {
    self.inner.max_frame_length()
  }

  pub fn set_max_frame_length(&mut self, val: usize)
  {
    self.inner.set_max_frame_length(val)
  }

  pub fn new() -> Self
  {
    Builder::default().into()
  }
}

impl Encoder for MsgPackCodec {
  type Item = rmpv::Value;
  type Error = std::io::Error;

  fn encode(&mut self, data: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error>
  {
    let mut buf = Vec::new();
    rmpv::encode::write_value(&mut buf, &data)?;
    self.inner.encode(buf.into(), dst)
  }
}

impl Decoder for MsgPackCodec {
  type Item = rmpv::Value;
  type Error = std::io::Error;

  fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>
  {
    use rmpv::decode::Error::*;
    use std::io::Cursor;

    match self.inner.decode(src)? {
      None => Ok(None),
      Some(complete_frame) => {
        let mut rdr = Cursor::new(complete_frame);

        match rmpv::decode::read_value(&mut rdr) {
          Ok(value) => Ok(Some(value)),
          Err(InvalidDataRead(error)) => Err(error),
          Err(InvalidMarkerRead(error)) => Err(error),
        }
      }
    }
  }
}

/// Builder-pattern structure
pub struct Builder {
  inner: BuilderLDC,
}

impl Builder {
  pub fn max_frame_length(&mut self, val: usize) -> &mut Self
  {
    self.inner.max_frame_length(val);
    self
  }

  pub fn length_field_length(&mut self, val: usize) -> &mut Self
  {
    self.inner.length_field_length(val);
    self
  }

  pub fn length_field_offset(&mut self, val: usize) -> &mut Self
  {
    self.inner.length_field_offset(val);
    self
  }
}

impl Default for Builder {
  fn default() -> Self
  {
    Self {
      inner: BuilderLDC::new(),
    }
  }
}

impl From<Builder> for MsgPackCodec {
  fn from(params: Builder) -> Self
  {
    Self {
      inner: params.inner.new_codec(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use bytes::buf::IntoBuf;
  // use tokio_io;

  #[test]
  fn test_simple_value()
  {
    let test_items = vec![
      (123.321f32, vec![0, 0, 0, 5, 202, 66, 246, 164, 90]),
    ];

    for (val, expected) in test_items.into_iter() {
      let val = rmpv::Value::from(val);

      let mut buf = Vec::with_capacity(100);
      rmpv::encode::write_value(&mut buf, &val).unwrap();
      assert_eq!(&buf[..], &expected[4..]);

      let mut codec = MsgPackCodec::new();
      let mut bytes = BytesMut::new();
      codec.encode(val.clone(), &mut bytes).unwrap();

      let bytebuff = bytes.into_buf().into_inner();
      assert_eq!(bytebuff, &expected);
    }
  }
}
