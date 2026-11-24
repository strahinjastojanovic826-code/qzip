use crate::error::QzipError;
use crate::qzip::Qzip;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use crate::quat::Quat;
use std::io;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct AsyncQzip;

impl AsyncQzip {
    pub fn compress_async(data: Vec<u8>) -> Receiver<Vec<u8>> {
        let (tx, rx) = channel();
        thread::spawn(move || {
            let compressed = Qzip::compress(&data);
            let _ = tx.send(compressed);
        });
        rx
    }

    pub fn decompress_async(compressed: Vec<u8>) -> Receiver<Result<Vec<u8>, QzipError>> {
        let (tx, rx) = channel();
        thread::spawn(move || {
            let decompressed = Qzip::decompress(&compressed);
            let _ = tx.send(decompressed);
        });
        rx
    }
    pub async fn compress_stream<R, W>(mut reader: R, mut writer: W) -> Result<(), QzipError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).await?;
    let compressed = Qzip::compress(&buffer);
    writer.write_all(&compressed).await?;
    writer.flush().await?;
    Ok(())
}
pub async fn decompress_stream<R, W>(mut reader: R, mut writer: W) -> Result<(), QzipError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut compressed = Vec::new();
    reader.read_to_end(&mut compressed).await?;
    let decompressed = Qzip::decompress(&compressed)?;
    writer.write_all(&decompressed).await?;
    writer.flush().await?;
    Ok(())
}
}

/// Offloads quaternary packing (4 Quats per byte) onto Tokio's blocking thread pool.
pub async fn pack_quats_blocking(quats: Vec<Quat>) -> Result<Vec<u8>, QzipError> {
    tokio::task::spawn_blocking(move || {
        let mut out = Vec::with_capacity(quats.len().div_ceil(4));
        for chunk in quats.chunks(4) {
            let q0 = chunk.first().copied().unwrap_or(Quat::Q0);
            let q1 = chunk.get(1).copied().unwrap_or(Quat::Q0);
            let q2 = chunk.get(2).copied().unwrap_or(Quat::Q0);
            let q3 = chunk.get(3).copied().unwrap_or(Quat::Q0);
            out.push(Quat::pack_four(q0, q1, q2, q3));
        }
        Ok(out)
    })
    .await
    .map_err(|e| QzipError::IoError(format!("spawn_blocking join error: {}", e)))?
}

/// Offloads quaternary unpacking into `Quat` symbols onto Tokio's blocking thread pool.
pub async fn unpack_quats_blocking(packed: Vec<u8>, quat_count: usize) -> Result<Vec<Quat>, QzipError> {
    tokio::task::spawn_blocking(move || Ok(Quat::unpack_quats_arch(&packed, quat_count)))
        .await
        .map_err(|e| QzipError::IoError(format!("spawn_blocking join error: {}", e)))?
}

/// Streaming `AsyncRead` over packed quaternary bytes (4 Quats per input byte -> 4 Quat bytes out).
pub struct AsyncQuatUnpackReader<R> {
    inner: R,
    staged: Vec<u8>,
    staged_pos: usize,
}

impl<R: AsyncRead + Unpin> AsyncQuatUnpackReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            staged: Vec::new(),
            staged_pos: 0,
        }
    }
}

impl<R: AsyncRead + Unpin> AsyncRead for AsyncQuatUnpackReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = &mut *self;

        while this.staged_pos >= this.staged.len() {
            let mut raw = [0u8; 1024];
            let mut read_buf = tokio::io::ReadBuf::new(&mut raw);
            match Pin::new(&mut this.inner).poll_read(cx, &mut read_buf) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Ready(Ok(())) => {
                    let filled = read_buf.filled();
                    if filled.is_empty() {
                        return Poll::Ready(Ok(()));
                    }
                    this.staged.clear();
                    this.staged_pos = 0;
                    this.staged.reserve(filled.len() * 4);
                    for &b in filled {
                        for q in Quat::unpack_four(b) {
                            this.staged.push(q.as_u8());
                        }
                    }
                }
            }
        }

        let avail = &this.staged[this.staged_pos..];
        let to_copy = avail.len().min(buf.remaining());
        buf.put_slice(&avail[..to_copy]);
        this.staged_pos += to_copy;

        Poll::Ready(Ok(()))
    }
}

/// Streaming `AsyncWrite` that packs incoming 2-bit quaternary values (0..=3) 4-per-byte
/// into the underlying async writer.
pub struct AsyncQuatPackWriter<W> {
    inner: W,
    quat_buf: [Quat; 4],
    quat_len: usize,
    out_buf: Vec<u8>,
    out_pos: usize,
}

impl<W: AsyncWrite + Unpin> AsyncQuatPackWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            quat_buf: [Quat::Q0; 4],
            quat_len: 0,
            out_buf: Vec::new(),
            out_pos: 0,
        }
    }

    fn flush_staged_out(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        while self.out_pos < self.out_buf.len() {
            match Pin::new(&mut self.inner).poll_write(cx, &self.out_buf[self.out_pos..]) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(err)) => return Poll::Ready(Err(err)),
                Poll::Ready(Ok(0)) => {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write packed quaternary bytes",
                    )));
                }
                Poll::Ready(Ok(n)) => self.out_pos += n,
            }
        }
        self.out_buf.clear();
        self.out_pos = 0;
        Poll::Ready(Ok(()))
    }
}

impl<W: AsyncWrite + Unpin> AsyncWrite for AsyncQuatPackWriter<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        let this = &mut *self;

        if let Poll::Ready(Err(err)) = this.flush_staged_out(cx) {
            return Poll::Ready(Err(err));
        }
        if this.out_pos < this.out_buf.len() {
            return Poll::Pending;
        }

        for &b in buf {
            this.quat_buf[this.quat_len] = Quat::from_u8(b);
            this.quat_len += 1;
            if this.quat_len == 4 {
                let packed = Quat::pack_four(
                    this.quat_buf[0],
                    this.quat_buf[1],
                    this.quat_buf[2],
                    this.quat_buf[3],
                );
                this.out_buf.push(packed);
                this.quat_len = 0;
            }
        }

        let _ = this.flush_staged_out(cx);
        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = &mut *self;
        if this.quat_len > 0 {
            let packed = Quat::pack_four(
                this.quat_buf[0],
                this.quat_buf[1],
                this.quat_buf[2],
                this.quat_buf[3],
            );
            this.out_buf.push(packed);
            this.quat_len = 0;
        }
        if let Poll::Ready(Err(err)) = this.flush_staged_out(cx) {
            return Poll::Ready(Err(err));
        }
        if this.out_pos < this.out_buf.len() {
            return Poll::Pending;
        }
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let this = &mut *self;
        if let Poll::Ready(pending_or_err) = Pin::new(&mut *this).poll_flush(cx) {
            if let Err(err) = pending_or_err {
                return Poll::Ready(Err(err));
            }
        }
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}