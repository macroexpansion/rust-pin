use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::FutureExt;
use pin_project::pin_project;
use tokio::{
    fs::File,
    io::{AsyncRead, AsyncReadExt, ReadBuf},
    time::{sleep, Instant, Sleep},
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

struct SlowReadPinBox<R> {
    reader: Pin<Box<R>>,
    sleep: Pin<Box<Sleep>>,
}

impl<R> SlowReadPinBox<R> {
    fn new(reader: R) -> Self {
        Self {
            reader: Box::pin(reader),
            sleep: Box::pin(tokio::time::sleep(Default::default())),
        }
    }
}

impl<R> AsyncRead for SlowReadPinBox<R>
where
    R: AsyncRead,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        match self.sleep.poll_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(_) => {
                self.sleep
                    .as_mut()
                    .reset(Instant::now() + Duration::from_secs(1));
                self.reader.as_mut().poll_read(cx, buf)
            }
        }
    }
}

#[pin_project]
struct SlowRead<R> {
    #[pin]
    reader: R,

    #[pin]
    sleep: Sleep,
}

impl<R> SlowRead<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            sleep: tokio::time::sleep(Default::default()),
        }
    }
}

impl<R> AsyncRead for SlowRead<R>
where
    R: AsyncRead + Unpin,
{
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        /* unsafe pin-project */
        // let (mut sleep, reader) = unsafe {
        //     let this = self.get_unchecked_mut();
        //     (
        //         Pin::new_unchecked(&mut this.sleep),
        //         Pin::new_unchecked(&mut this.reader),
        //     )
        // };
        // match sleep.as_mut().poll(cx) {
        //     Poll::Ready(_) => {
        //         sleep.reset(Instant::now() + Duration::from_secs(1));
        //         reader.poll_read(cx, buf)
        //     }
        //     Poll::Pending => Poll::Pending,
        // }

        /* safe pin-project */
        let mut this = self.project();
        match this.sleep.as_mut().poll(cx) {
            Poll::Ready(_) => {
                this.sleep.reset(Instant::now() + Duration::from_secs(1));
                this.reader.poll_read(cx, buf)
            }
            Poll::Pending => Poll::Pending,
        }

        /* unsafe map_unchecked_mut */
        // let sleep = unsafe { self.as_mut().map_unchecked_mut(|this| &mut this.sleep) };
        // match sleep.poll(cx) {
        //     Poll::Ready(_) => {
        //         let sleep = unsafe { self.as_mut().map_unchecked_mut(|this| &mut this.sleep) };
        //         sleep.reset(Instant::now() + Duration::from_secs(1));
        //         let reader = unsafe { self.as_mut().map_unchecked_mut(|this| &mut this.reader) };
        //         reader.poll_read(cx, buf)
        //     }
        //     Poll::Pending => Poll::Pending,
        // }
    }
}

async fn slow_read() -> Result<(), Box<tokio::io::Error>> {
    let mut buf = vec![0u8; 128 * 1024];
    let reader = File::open("/dev/urandom").await?;
    let mut slow_reader = SlowRead::new(reader);
    let mut slow_reader = unsafe { Pin::new_unchecked(&mut slow_reader) };
    let before = Instant::now();
    slow_reader.read_exact(&mut buf).await?;
    println!("Read {} bytes in {:?}", buf.len(), before.elapsed());
    Ok(())
}

async fn slow_read_pin_box() -> Result<(), Box<tokio::io::Error>> {
    let mut buf = vec![0u8; 128 * 1024];
    let reader = File::open("/dev/urandom").await?;
    let mut slow_reader = SlowReadPinBox::new(reader);
    let before = Instant::now();
    slow_reader.read_exact(&mut buf).await?;
    println!("Read {} bytes in {:?}", buf.len(), before.elapsed());
    Ok(())
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let future = SleepFuture::new();

    let t = tokio::spawn(future);
    let _ = t.await;

    // slow_read_pin_box().await.unwrap();
    slow_read().await.unwrap();
}
