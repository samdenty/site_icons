use super::IconSize;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
  cmp::Ordering,
  error::Error,
  fmt::{self, Display},
  ops::{Deref, DerefMut},
};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct IconSizes(Vec<IconSize>);

impl Display for IconSizes {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    f.write_str(&self.0.iter().join(" "))
  }
}

impl IconSizes {
  pub fn new() -> Self {
    IconSizes(Vec::new())
  }

  pub fn from_str(sizes_str: &str) -> Result<IconSizes, Box<dyn Error>> {
    let size_strs = sizes_str.split(" ");

    let mut sizes = IconSizes::new();
    for size in size_strs {
      if let Ok(size) = serde_json::from_value(Value::String(size.to_string())) {
        sizes.push(size);
      }
    }

    if sizes.is_empty() {
      return Err("must contain a size".into());
    }

    sizes.sort();

    Ok(sizes)
  }

  pub fn add_size(&mut self, width: u32, height: u32) {
    self.push(IconSize::new(width, height))
  }

  pub fn largest(&self) -> &IconSize {
    &self.0[0]
  }

  pub fn into_largest(self) -> IconSize {
    self.0.into_iter().next().unwrap()
  }
}

impl Deref for IconSizes {
  type Target = Vec<IconSize>;
  fn deref(&self) -> &Vec<IconSize> {
    &self.0
  }
}

impl DerefMut for IconSizes {
  fn deref_mut(&mut self) -> &mut Vec<IconSize> {
    &mut self.0
  }
}

impl Ord for IconSizes {
  fn cmp(&self, other: &Self) -> Ordering {
    self.largest().cmp(&other.largest())
  }
}

impl PartialOrd for IconSizes {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}
