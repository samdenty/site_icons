macro_rules! selector {
  ($($selector:expr),+ $(,)?) => {{
    static RE: once_cell::sync::OnceCell<scraper::Selector> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| scraper::Selector::parse(join_with!(",", $($selector),+)).unwrap())
  }};
}

macro_rules! join_with {
  ($pattern:literal,$first:expr$(, $($rest:expr),*)? $(,)?) => {
    concat!($first$(, $($pattern, $rest),*)?)
  };
}

macro_rules! regex {
    ($re:literal $(,)?) => {{
        static RE: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        RE.get_or_init(|| regex::Regex::new($re).unwrap())
    }};
}

macro_rules! assert_slice_eq {
  ($cur:expr, $offset:expr, $slice:expr, $($arg:tt)+) => {{
    if !super::slice_eq($cur, $offset, $slice)? {
      return Err(format!($($arg)+).into());
    }
  }};
}
