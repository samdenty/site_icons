use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
use std::borrow::Cow;

const DATA_URI: &AsciiSet = &CONTROLS
  .add(b'\r')
  .add(b'\n')
  .add(b'%')
  .add(b'#')
  .add(b'(')
  .add(b')')
  .add(b'<')
  .add(b'>')
  .add(b'?')
  .add(b'[')
  .add(b'\\')
  .add(b']')
  .add(b'^')
  .add(b'`')
  .add(b'{')
  .add(b'|')
  .add(b'}');

pub fn encode_svg(svg: &str) -> String {
  // add namespace
  let encoded = if !svg.contains("http://www.w3.org/2000/svg") {
    regex!("<svg").replace(svg, "<svg xmlns='http://www.w3.org/2000/svg'")
  } else {
    svg.into()
  };

  // use single quotes instead of double to avoid encoding.
  let mut encoded = regex!("\"").replace_all(&encoded, "'");

  // remove a fill=none attribute
  if let Some(captures) = regex!("^[^>]+fill='?(none)'?").captures(&encoded) {
    let index = captures.get(1).unwrap();
    let mut result = String::new();
    for (i, c) in encoded.chars().enumerate() {
      if i < index.start() || i >= index.end() {
        result.push(c);
      }
    }
    encoded = Cow::from(result);
  }

  // remove whitespace
  let encoded = regex!(r">\s{1,}<").replace_all(&encoded, "><");
  let encoded = regex!(r"\s{2,}").replace_all(&encoded, " ");

  let encoded = utf8_percent_encode(&encoded, DATA_URI);

  format!("data:image/svg+xml,{}", encoded)
}
