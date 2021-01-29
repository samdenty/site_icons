#[macro_export]
macro_rules! selector {
  ($($selector:expr),+ $(,)?) => {{
    static RE: once_cell::sync::OnceCell<scraper::Selector> = once_cell::sync::OnceCell::new();
    RE.get_or_init(|| scraper::Selector::parse(crate::join!(",", $($selector),+)).unwrap())
  }};
}

#[macro_export]
macro_rules! join {
  ($pattern:literal,$first:expr$(, $($rest:expr),*)? $(,)?) => {
    concat!($first$(, $($pattern, $rest),*)?)
  };
}
