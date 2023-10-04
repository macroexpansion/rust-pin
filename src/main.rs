use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::FutureExt;
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, ReadBuf},
    time::{sleep, Sleep},
};

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

struct SlowRead<R> {
    reader: Pin<Box<R>>,
}

impl<R> SlowRead<R> {
    fn new(reader: R) -> Self {
        Self {
            reader: Box::pin(reader),
        }
    }
}

impl<R> AsyncRead for SlowRead<R>
where
    R: AsyncRead,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.reader.as_mut().poll_read(cx, buf)
    }
}

async fn slow_read() -> Result<(), Box<tokio::io::Error>> {
    let mut buf = vec![0u8; 128 * 1024];
    let reader = File::open("/dev/urandom").await?;
    let mut slow_reader = SlowRead::new(reader);
    slow_reader.read_exact(&mut buf).await?;
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let future = SleepFuture::new();

    let t = tokio::spawn(future);
    let _ = t.await;

    slow_read().await.unwrap();
}
