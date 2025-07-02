use crate::ffi::new_ready_promise_void;
use crate::Error;

type Result<T> = std::io::Result<T>;

/// Async stream of zeros of a given size
pub struct ZeroStream(usize);

impl ZeroStream {
    pub async fn try_read(&mut self, buffer: &mut [u8], min_bytes: usize) -> Result<usize> {
        let mut n = 0;
        while n < min_bytes {
            let k = futures::AsyncReadExt::read(self, &mut buffer[n..]).await?;
            if k == 0 {
                break;
            }
            n += k;
        }
        let _ = new_ready_promise_void().await.map_err(Error::other)?;
        Ok(n)
    }
}

pub fn new_zero_stream(size: usize) -> Box<ZeroStream> {
    Box::new(ZeroStream(size))
}

impl futures::io::AsyncRead for ZeroStream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let this = self.get_mut();
        let n = std::cmp::min(this.0, buf.len());
        this.0 -= n;
        buf[..n].fill(0);
        std::task::Poll::Ready(Ok(n))
    }
}

#[cfg(test)]
mod tests {
    use futures::executor::LocalPool;

    use super::*;

    pub fn run_local<Fut>(future: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        let mut pool = LocalPool::new();
        pool.run_until(future)
    }

    #[test]
    fn read_shorter_than_length() {
        let mut stream = ZeroStream(5);
        let mut buffer = [0; 3];
        assert_eq!(run_local(stream.try_read(&mut buffer, 3)).unwrap(), 3);
        assert_eq!(stream.0, 2);
    }

    #[test]
    fn read_longer_than_length() {
        let mut stream = ZeroStream(3);
        let mut buffer = [0; 5];
        assert_eq!(run_local(stream.try_read(&mut buffer, 5)).unwrap(), 3);
        assert_eq!(stream.0, 0);
    }

    #[test]
    fn try_read_shorter_than_length() {
        let mut stream = ZeroStream(5);
        let mut buffer = [0; 3];
        assert_eq!(run_local(stream.try_read(&mut buffer, 3)).unwrap(), 3);
        assert_eq!(stream.0, 2);
    }

    #[test]
    fn try_read_longer_than_length() {
        let mut stream = ZeroStream(3);
        let mut buffer = [0; 5];
        assert_eq!(run_local(stream.try_read(&mut buffer, 5)).unwrap(), 3);
        assert_eq!(stream.0, 0);
    }
}
