use crate::error::QzipError;
use crate::qzip::Qzip;
use std::sync::mpsc::{channel, Receiver};
use std::thread;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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
