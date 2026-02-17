use chia_wallet_sdk::{chia::protocol::BytesImpl, prelude::*};

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

impl Convert<u8> for i64 {
    fn convert(self) -> Result<u8> {
        Ok(self.try_into()?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_to_fixed_array() {
        let v: Vec<u8> = vec![1, 2, 3, 4];
        let result: [u8; 4] = v.convert().unwrap();
        assert_eq!(result, [1, 2, 3, 4]);
    }

    #[test]
    fn vec_to_fixed_array_wrong_length() {
        let v: Vec<u8> = vec![1, 2, 3];
        let result: std::result::Result<[u8; 4], _> = v.convert();
        assert!(result.is_err());
    }

    #[test]
    fn i64_to_u32() {
        let v: i64 = 42;
        let result: u32 = v.convert().unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn i64_to_u32_max() {
        let v: i64 = u32::MAX as i64;
        let result: u32 = v.convert().unwrap();
        assert_eq!(result, u32::MAX);
    }

    #[test]
    fn negative_i64_to_u32_fails() {
        let v: i64 = -1;
        let result: std::result::Result<u32, _> = v.convert();
        assert!(result.is_err());
    }

    #[test]
    fn i64_to_u64() {
        let v: i64 = 1_000_000;
        let result: u64 = v.convert().unwrap();
        assert_eq!(result, 1_000_000);
    }

    #[test]
    fn i64_to_u8() {
        let v: i64 = 255;
        let result: u8 = v.convert().unwrap();
        assert_eq!(result, 255);
    }

    #[test]
    fn i64_to_u8_overflow() {
        let v: i64 = 256;
        let result: std::result::Result<u8, _> = v.convert();
        assert!(result.is_err());
    }

    #[test]
    fn i64_to_u16() {
        let v: i64 = 65535;
        let result: u16 = v.convert().unwrap();
        assert_eq!(result, 65535);
    }

    #[test]
    fn option_some_converts() {
        let v: Option<i64> = Some(42);
        let result: Option<u32> = v.convert().unwrap();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn option_none_converts() {
        let v: Option<i64> = None;
        let result: Option<u32> = v.convert().unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn vec_to_u64_via_be_bytes() {
        let v: Vec<u8> = 1000u64.to_be_bytes().to_vec();
        let result: u64 = v.convert().unwrap();
        assert_eq!(result, 1000);
    }

    #[test]
    fn vec_to_u128_via_be_bytes() {
        let v: Vec<u8> = 999u128.to_be_bytes().to_vec();
        let result: u128 = v.convert().unwrap();
        assert_eq!(result, 999);
    }
}
