use crate::quat::Quat;
use crate::error::QzipError;

pub trait BinarySerializable {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Result<Self, QzipError> where Self: Sized;
}

impl BinarySerializable for Vec<Quat> {
    fn to_bytes(&self) -> Vec<u8> {
        self.iter().map(|q| q.as_u8()).collect()
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, QzipError> {
        if bytes.is_empty() {
            return Err(QzipError::EmptyData);
        }
        Ok(bytes.iter().map(|&b| Quat::from_u8(b)).collect())
    }
}