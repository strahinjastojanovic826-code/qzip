use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QzipError {
    EmptyData,
    CorruptStream,
    InvalidMagic,
    ChecksumMismatch,
    IoError(String),
}

impl fmt::Display for QzipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QzipError::EmptyData => write!(f, "Ulazni podaci su prazni"),
            QzipError::CorruptStream => write!(f, "Kompresovani tok je oštećen ili nevažeći"),
            QzipError::InvalidMagic => write!(f, "Neispravno zaglavlje (Magic header)"),
            QzipError::ChecksumMismatch => write!(f, "Netačna CRC32 kontrolna suma (oštećenje podataka)"),
            QzipError::IoError(msg) => write!(f, "I/O greška: {}", msg),
        }
    }
}

impl std::error::Error for QzipError {}

impl From<std::io::Error> for QzipError {
    fn from(err: std::io::Error) -> Self {
        QzipError::IoError(err.to_string())
    }
}