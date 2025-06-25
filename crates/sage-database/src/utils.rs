use chia::{
    bls::{PublicKey, Signature},
    protocol::{BytesImpl, Program},
};

use crate::{DatabaseError, Result};

pub trait Convert<T> {
    fn convert(self) -> Result<T>;
}

impl<const N: usize> Convert<[u8; N]> for Vec<u8> {
    fn convert(self) -> Result<[u8; N]> {
        let length = self.len();
        self.try_into()
            .map_err(|_| DatabaseError::InvalidLength(length, N))
    }
}

impl<const N: usize> Convert<BytesImpl<N>> for Vec<u8> {
    fn convert(self) -> Result<BytesImpl<N>> {
        Ok(BytesImpl::new(self.convert()?))
    }
}

impl Convert<PublicKey> for Vec<u8> {
    fn convert(self) -> Result<PublicKey> {
        Ok(PublicKey::from_bytes(&self.convert()?)?)
    }
}

impl Convert<Signature> for Vec<u8> {
    fn convert(self) -> Result<Signature> {
        Ok(Signature::from_bytes(&self.convert()?)?)
    }
}

impl Convert<u64> for Vec<u8> {
    fn convert(self) -> Result<u64> {
        Ok(u64::from_be_bytes(self.convert()?))
    }
}

impl Convert<u128> for Vec<u8> {
    fn convert(self) -> Result<u128> {
        Ok(u128::from_be_bytes(self.convert()?))
    }
}

impl Convert<u16> for i64 {
    fn convert(self) -> Result<u16> {
        Ok(self.try_into()?)
    }
}

impl Convert<u32> for i64 {
    fn convert(self) -> Result<u32> {
        Ok(self.try_into()?)
    }
}

impl Convert<u64> for i64 {
    fn convert(self) -> Result<u64> {
        Ok(self.try_into()?)
    }
}

impl<T, U> Convert<Option<T>> for Option<U>
where
    U: Convert<T>,
{
    fn convert(self) -> Result<Option<T>> {
        self.map(U::convert).transpose()
    }
}
