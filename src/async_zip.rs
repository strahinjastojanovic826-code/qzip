use crate::error::QzipError;
use crate::qzip::Qzip;
use std::sync::mpsc::{channel, Receiver};
use std::thread;

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
}