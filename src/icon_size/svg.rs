use super::IconSize;
use futures::prelude::*;
use lol_html::{element, errors::RewritingError, HtmlRewriter, Settings};
use std::{
  error::Error,
  fmt::{self, Display},
};

#[derive(Debug)]
struct SizeResult(Option<IconSize>);

impl Display for SizeResult {
  fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
    Ok(())
  }
}

impl Error for SizeResult {}

fn parse_size<S: ToString>(size: S) -> Option<u32> {
  size
    .to_string()
    .parse::<f64>()
    .ok()
    .map(|size| size.round() as u32)
}

pub async fn get_svg_size<R: AsyncRead + Unpin>(
  first_bytes: &[u8; 2],
  reader: &mut R,
) -> Result<Option<IconSize>, Box<dyn Error>> {
  let mut rewriter = HtmlRewriter::new(
    Settings {
      element_content_handlers: vec![
        // Rewrite insecure hyperlinks
        element!("svg", |el| {
          let viewbox = el.get_attribute("viewbox");

          let width = el.get_attribute("width").and_then(parse_size);
          let height = el.get_attribute("height").and_then(parse_size);

          Err(Box::new(SizeResult(
            if let (Some(width), Some(height)) = (width, height) {
              Some(IconSize::new(width, height))
            } else if let Some(viewbox) = viewbox {
              regex!(r"^\d+\s+\d+\s+(\d+\.?[\d]?)\s+(\d+\.?[\d]?)")
                .captures(&viewbox)
                .map(|captures| {
                  let width = parse_size(captures.get(1).unwrap().as_str()).unwrap();
                  let height = parse_size(captures.get(2).unwrap().as_str()).unwrap();
                  IconSize::new(width, height)
                })
            } else {
              None
            },
          )))
        }),
      ],
      ..Settings::default()
    },
    |_: &[u8]| {},
  );

  rewriter.write(first_bytes)?;

  let mut buffer = [0; 100];

  loop {
    let n = reader.read(&mut buffer).await?;
    if n == 0 {
      return Err("invalid svg".into());
    }

    match rewriter.write(&buffer[..n]) {
      Err(RewritingError::ContentHandlerError(result)) => {
        let result = result.downcast::<SizeResult>().unwrap();

        return Ok(result.0);
      }

      result => result?,
    }
  }
}
