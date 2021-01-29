use super::{png::get_png_sizes, IconSizes};
use byteorder::{LittleEndian, ReadBytesExt};
use futures::prelude::*;
use std::{
  error::Error,
  io::{Cursor, Seek, SeekFrom},
};

const ICO_TYPE: u16 = 1;
const INDEX_SIZE: u16 = 16;

pub async fn get_ico_sizes<R: AsyncRead + Unpin>(
  reader: &mut R,
) -> Result<IconSizes, Box<dyn Error>> {
  let mut offset = 0;
  let mut header = [0; 6];
  reader.read_exact(&mut header).await?;
  offset += header.len();
  let mut header = Cursor::new(header);

  let header_type = header.read_u16::<LittleEndian>()?;
  let icon_type = header.read_u16::<LittleEndian>()?;

  if header_type != 0 || icon_type != ICO_TYPE {
    return Err("bad header".into());
  }

  let icon_count = header.read_u16::<LittleEndian>()?;

  let mut data = vec![0; (icon_count * INDEX_SIZE) as usize];
  reader.read_exact(&mut data).await?;
  offset += data.len();
  let mut data = Cursor::new(data);

  let mut sizes = IconSizes::new();
  for i in 0..icon_count {
    data.seek(SeekFrom::Start((INDEX_SIZE * i) as _))?;

    let width = data.read_u8()?;
    let height = data.read_u8()?;

    if width == 0 && height == 0 {
      data.seek(SeekFrom::Current(10))?;
      let image_offset = data.read_u32::<LittleEndian>()?;

      let mut data = vec![0; image_offset as usize - offset];
      reader.read_exact(&mut data).await?;
      offset += data.len();

      let size = get_png_sizes(reader).await;
      if let Ok(size) = size {
        sizes.push(size);
      }
    } else {
      sizes.add_size(width as _, height as _)
    }
  }

  sizes.sort();

  Ok(sizes)
}
