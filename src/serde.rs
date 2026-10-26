use crate::quat::Quat;
use crate::error::QzipError;
use serde::{Serialize, Deserialize, Serializer};

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

impl Serialize for Quat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer, // Ovdje definišeš da je S serijalizator
    {
        serializer.serialize_u8(self.as_u8())
    }
}

impl<'de> Deserialize<'de> for Quat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>, // Ovdje definišeš da je D deserijalizator
    {
        let val = u8::deserialize(deserializer)?;
        Ok(Quat::from_u8(val))
    }
}