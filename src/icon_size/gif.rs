use super::IconSize;
use byteorder::{LittleEndian, ReadBytesExt};
use futures::prelude::*;
use std::{
  error::Error,
  io::{Cursor, Seek, SeekFrom},
};

pub async fn get_gif_size<R: AsyncRead + Unpin>(
  reader: &mut R,
) -> Result<IconSize, Box<dyn Error>> {
  let mut header = [0; 8];
  reader.read_exact(&mut header).await?;
  let header = &mut Cursor::new(header);

  assert_slice_eq!(header, 0, b"F8", "bad header");

  header.seek(SeekFrom::Start(4))?;

  let width = header.read_u16::<LittleEndian>()? as u32;
  let height = header.read_u16::<LittleEndian>()? as u32;

  Ok(IconSize::new(width, height))
}
