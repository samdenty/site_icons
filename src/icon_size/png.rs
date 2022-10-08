use super::IconSize;
use byteorder::{BigEndian, ReadBytesExt};
use futures::prelude::*;
use std::{error::Error, io::Cursor};

pub async fn get_png_size<R: AsyncRead + Unpin>(
  reader: &mut R,
) -> Result<IconSize, Box<dyn Error>> {
  let mut header = [0; 22];
  reader.read_exact(&mut header).await?;
  let header = &mut Cursor::new(header);

  assert_slice_eq!(header, 0, b"PNG\r\n\x1a\n", "bad header");
  assert_slice_eq!(header, 10, b"IHDR", "bad header");

  let width = header.read_u32::<BigEndian>()?;
  let height = header.read_u32::<BigEndian>()?;

  Ok(IconSize::new(width, height))
}
