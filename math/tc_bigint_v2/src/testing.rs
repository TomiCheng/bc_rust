//! Generators the tests of the random integers share.

use core::convert::Infallible;
use core::fmt;

use rand_core::TryRng;

/// Gives the bytes 1, 2, 3 and on, so that what a draw takes shows.
pub(crate) struct Counting(pub(crate) u8);

impl TryRng for Counting {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        let mut bytes = [0; 4];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        let mut bytes = [0; 8];
        self.try_fill_bytes(&mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        for byte in dst {
            self.0 = self.0.wrapping_add(1);
            *byte = self.0;
        }
        Ok(())
    }
}

/// A xorshift generator, for draws that look random.
pub(crate) struct Xorshift(pub(crate) u64);

impl TryRng for Xorshift {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        Ok(self.try_next_u64()? as u32)
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        Ok(self.0)
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        for chunk in dst.chunks_mut(8) {
            let word = self.try_next_u64()?.to_le_bytes();
            chunk.copy_from_slice(&word[..chunk.len()]);
        }
        Ok(())
    }
}

/// A generator that has failed.
pub(crate) struct Broken;

#[derive(Debug, PartialEq)]
pub(crate) struct Failure;

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the generator failed")
    }
}

impl core::error::Error for Failure {}

impl TryRng for Broken {
    type Error = Failure;

    fn try_next_u32(&mut self) -> Result<u32, Failure> {
        Err(Failure)
    }

    fn try_next_u64(&mut self) -> Result<u64, Failure> {
        Err(Failure)
    }

    fn try_fill_bytes(&mut self, _: &mut [u8]) -> Result<(), Failure> {
        Err(Failure)
    }
}

/// The bytes 1 to `count`, low first, as one value.
pub(crate) fn counted(count: u8) -> u128 {
    (1..=count)
        .rev()
        .fold(0, |value, byte| value << 8 | u128::from(byte))
}
