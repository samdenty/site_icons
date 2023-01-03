use std::{
  pin::Pin,
  task::{Context, Poll},
};

use futures::Future;

pub async fn poll_in_background<F, B, FO, BO>(future: F, background_future: B) -> FO
where
  F: Future<Output = FO> + Unpin,
  B: Future<Output = BO> + Unpin,
{
  struct BackgroundPoller<F, B> {
    future: F,
    background_future: B,
  }

  impl<F, B, FO, BO> Future for BackgroundPoller<F, B>
  where
    F: Future<Output = FO> + Unpin,
    B: Future<Output = BO> + Unpin,
  {
    type Output = FO;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      let this = self.get_mut();

      let result = Pin::new(&mut this.future).poll(cx);

      if result.is_pending() {
        let _ = Pin::new(&mut this.background_future).poll(cx);
      }

      result
    }
  }

  BackgroundPoller {
    future,
    background_future,
  }
  .await
}
