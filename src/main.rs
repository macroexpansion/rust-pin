use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::FutureExt;
use tokio::time::{sleep, Sleep};

struct SleepFuture {
    sleep: Pin<Box<Sleep>>,
}

impl SleepFuture {
    fn new() -> Self {
        Self {
            sleep: Box::pin(sleep(Duration::from_secs(1))),
        }
    }
}

impl Future for SleepFuture {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        /* using Pin */
        // let sleep = Pin::new(&mut self.sleep);
        // sleep.poll(cx)

        /* using as_mut() */
        // self.sleep.as_mut().poll(cx)

        /* poll_unpin */
        self.sleep.poll_unpin(cx)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let future = SleepFuture::new();

    let t = tokio::spawn(future);
    let _ = t.await;
}
