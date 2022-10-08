use super::IconSize;
use byteorder::BigEndian;
use futures::prelude::*;
use std::{error::Error, io::Cursor};
use tokio_futures_byteorder::AsyncReadBytesExt;

pub async fn get_jpeg_size<R: AsyncRead + Unpin>(
  reader: &mut R,
) -> Result<IconSize, Box<dyn Error>> {
  let mut marker = [0; 2];
  let mut depth = 0i32;

  loop {
    // Read current marker (FF XX)
    reader.read_exact(&mut marker).await?;

    if marker[0] != 0xFF {
      //  Did not read a marker. Assume image is corrupt.
      return Err("invalid jpeg".into());
    }

    let page = marker[1];

    //  Check for valid SOFn markers. C4, C8, and CC aren't dimension markers.
    if (page >= 0xC0 && page <= 0xC3)
      || (page >= 0xC5 && page <= 0xC7)
      || (page >= 0xC9 && page <= 0xCB)
      || (page >= 0xCD && page <= 0xCF)
    {
      //  Only get outside image size
      if depth == 0 {
        //  Correct marker, go forward 3 bytes so we're at height offset
        reader.read_exact(&mut [0; 3]).await?;
        break;
      }
    } else if page == 0xD8 {
      depth += 1;
    } else if page == 0xD9 {
      depth -= 1;
      if depth < 0 {
        return Err("invalid jpeg".into());
      }
    }

    //  Read the marker length and skip over it entirely
    let page_size = reader.read_u16::<BigEndian>().await? as i64;
    reader
      .read_exact(&mut vec![0; (page_size - 2) as usize])
      .await?;
  }

  let height = reader.read_u16::<BigEndian>().await?;
  let width = reader.read_u16::<BigEndian>().await?;

  Ok(IconSize::new(width as _, height as _))
}
