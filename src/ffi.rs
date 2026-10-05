use crate::qzip::Qzip;
use crate::error::QzipError;
use std::panic::catch_unwind;
use std::slice;

/// FFI Status codes returned to C / C++ / C#
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum QzipStatus {
    Success = 0,
    NullPointer = 1,
    BufferTooSmall = 2,
    EmptyData = 3,
    CorruptStream = 4,
    InvalidMagic = 5,
    ChecksumMismatch = 6,
    IoError = 7,
    PanicOccurred = 8,
}

impl From<QzipError> for QzipStatus {
    fn from(err: QzipError) -> Self {
        match err {
            QzipError::EmptyData => QzipStatus::EmptyData,
            QzipError::CorruptStream => QzipStatus::CorruptStream,
            QzipError::InvalidMagic => QzipStatus::InvalidMagic,
            QzipError::ChecksumMismatch => QzipStatus::ChecksumMismatch,
            QzipError::IoError(_) => QzipStatus::IoError,
        }
    }
}

/// Compresses input bytes into caller-allocated output buffer.
///
/// - If `out_ptr` is NULL, `out_len` will be updated with the required buffer size.
/// - Returns `QzipStatus::BufferTooSmall` if `*out_len` is insufficient.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qzip_compress(
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> QzipStatus {
    if in_ptr.is_null() || out_len.is_null() {
        return QzipStatus::NullPointer;
    }

    let result = catch_unwind(|| {
        let input = slice::from_raw_parts(in_ptr, in_len);
        let compressed = Qzip::compress(input);
        let required_len = compressed.len();

        if out_ptr.is_null() {
            *out_len = required_len;
            return QzipStatus::Success;
        }

        if *out_len < required_len {
            *out_len = required_len;
            return QzipStatus::BufferTooSmall;
        }

        std::ptr::copy_nonoverlapping(compressed.as_ptr(), out_ptr, required_len);
        *out_len = required_len;

        QzipStatus::Success
    });

    result.unwrap_or(QzipStatus::PanicOccurred)
}

/// Decompresses input bytes into caller-allocated output buffer.
///
/// - If `out_ptr` is NULL, `out_len` will be updated with the required buffer size.
/// - Returns `QzipStatus::BufferTooSmall` if `*out_len` is insufficient.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn qzip_decompress(
    in_ptr: *const u8,
    in_len: usize,
    out_ptr: *mut u8,
    out_len: *mut usize,
) -> QzipStatus {
    if in_ptr.is_null() || out_len.is_null() {
        return QzipStatus::NullPointer;
    }

    let result = catch_unwind(|| {
        let input = slice::from_raw_parts(in_ptr, in_len);

        match Qzip::decompress(input) {
            Ok(decompressed) => {
                let required_len = decompressed.len();

                if out_ptr.is_null() {
                    *out_len = required_len;
                    return QzipStatus::Success;
                }

                if *out_len < required_len {
                    *out_len = required_len;
                    return QzipStatus::BufferTooSmall;
                }

                std::ptr::copy_nonoverlapping(decompressed.as_ptr(), out_ptr, required_len);
                *out_len = required_len;

                QzipStatus::Success
            }
            Err(err) => QzipStatus::from(err),
        }
    });

    result.unwrap_or(QzipStatus::PanicOccurred)
}