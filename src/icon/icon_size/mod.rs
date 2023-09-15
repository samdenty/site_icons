mod gif;
mod ico;
mod icon_sizes;
mod jpeg;
mod png;
mod svg;

pub use gif::*;
pub use ico::*;
pub use icon_sizes::*;
pub use jpeg::*;
pub use png::*;
pub use svg::*;

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::{
    cmp::{self, Ordering},
    error::Error,
    fmt::{self, Display},
    io::{Read, Seek, SeekFrom},
};

#[serde_as]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IconSize {
    pub width: u32,
    pub height: u32,
}

impl Display for IconSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

impl IconSize {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn max_rect(&self) -> u32 {
        cmp::max(self.width, self.height)
    }
}

impl Ord for IconSize {
    fn cmp(&self, other: &Self) -> Ordering {
        other.max_rect().cmp(&self.max_rect())
    }
}

impl PartialOrd for IconSize {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for IconSize {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for IconSize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: String = Deserialize::deserialize(deserializer)?;

        let mut split = value.split("x");
        let width = split
            .next()
            .ok_or(de::Error::custom("expected width"))?
            .parse()
            .map_err(de::Error::custom)?;

        let height = split
            .next()
            .ok_or(de::Error::custom("expected height"))?
            .parse()
            .map_err(de::Error::custom)?;

        Ok(IconSize::new(width, height))
    }
}

fn slice_eq<T: Read + Seek + Unpin>(
    cur: &mut T,
    offset: u64,
    slice: &[u8],
) -> Result<bool, Box<dyn Error>> {
    cur.seek(SeekFrom::Start(offset))?;
    let mut buffer = vec![0; slice.len()];
    cur.read_exact(&mut buffer)?;
    Ok(buffer == slice)
}
