use super::IconSize;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    error::Error,
    fmt::{self, Display},
    ops::Deref,
};
use vec1::{vec1, Vec1};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
/// Icon sizes, ordered from largest to smallest
pub struct IconSizes(Vec1<IconSize>);

impl Display for IconSizes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.iter().join(" "))
    }
}

impl IconSizes {
    pub fn add_size(&mut self, size: IconSize) {
        match self.0.binary_search(&size) {
            Ok(_) => {}
            Err(pos) => self.0.insert(pos, size),
        }
    }

    pub fn largest(&self) -> &IconSize {
        self.0.first()
    }
}

impl TryFrom<&str> for IconSizes {
    type Error = Box<dyn Error>;

    fn try_from(sizes_str: &str) -> Result<Self, Self::Error> {
        let size_strs = sizes_str.split(' ');

        let mut sizes = Vec::new();
        for size in size_strs {
            if let Ok(size) = serde_json::from_value(Value::String(size.to_string())) {
                sizes.push(size);
            }
        }

        Ok(sizes.try_into()?)
    }
}

impl TryFrom<&String> for IconSizes {
    type Error = Box<dyn Error>;

    fn try_from(sizes_str: &String) -> Result<Self, Self::Error> {
        IconSizes::try_from(sizes_str.as_str())
    }
}

impl TryFrom<String> for IconSizes {
    type Error = Box<dyn Error>;

    fn try_from(sizes_str: String) -> Result<Self, Self::Error> {
        IconSizes::try_from(sizes_str.as_str())
    }
}

impl Deref for IconSizes {
    type Target = Vec1<IconSize>;
    fn deref(&self) -> &Vec1<IconSize> {
        &self.0
    }
}

impl IntoIterator for IconSizes {
    type Item = IconSize;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Ord for IconSizes {
    fn cmp(&self, other: &Self) -> Ordering {
        self.largest().cmp(other.largest())
    }
}

impl PartialOrd for IconSizes {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl TryFrom<Vec<IconSize>> for IconSizes {
    type Error = String;

    fn try_from(mut vec: Vec<IconSize>) -> Result<Self, Self::Error> {
        vec.sort();

        Ok(IconSizes(
            vec.try_into().map_err(|_| "must contain a size")?,
        ))
    }
}

impl From<IconSize> for IconSizes {
    fn from(size: IconSize) -> Self {
        IconSizes(vec1![size])
    }
}
