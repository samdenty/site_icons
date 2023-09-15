use std::error::Error;

use futures::{AsyncRead, AsyncReadExt as _};

use super::IconSize;

async fn read_u16_be<R: AsyncRead + Unpin>(reader: &mut R) -> Result<u16, Box<dyn Error>> {
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf).await?;
    Ok(u16::from_be_bytes(buf))
}

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
        if (0xC0..=0xC3).contains(&page)
            || (0xC5..=0xC7).contains(&page)
            || (0xC9..=0xCB).contains(&page)
            || (0xCD..=0xCF).contains(&page)
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
        let page_size = read_u16_be(reader).await? as i64;
        reader
            .read_exact(&mut vec![0; (page_size - 2) as usize])
            .await?;
    }

    let height = read_u16_be(reader).await?;
    let width = read_u16_be(reader).await?;

    Ok(IconSize::new(width as _, height as _))
}
