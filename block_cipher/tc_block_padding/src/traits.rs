pub trait BlockCipherPadding {
    /// The failure type returned by padding operations.
    type Error: core::error::Error;

    /// Pads `block[position..]` and returns the number of padding bytes added.
    ///
    /// `block` is one complete cipher block whose first `position` bytes hold
    /// the remaining message. Implementations overwrite every byte from
    /// `position` to the end of the block, so a `position` equal to the block
    /// length adds no bytes and leaves the block unchanged.
    ///
    /// The receiver is mutable because schemes that draw padding from a random
    /// generator advance that generator here.
    ///
    /// # Errors
    ///
    /// Returns an error when `position` is greater than the block length.
    fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error>;

    /// Returns the number of padding bytes at the end of `block`.
    ///
    /// The message occupies `block.len() - pad_count(block)` bytes. Callers
    /// must treat the result as untrusted length information until the message
    /// itself has been authenticated.
    ///
    /// # Errors
    ///
    /// Self-describing schemes return an error when the trailing bytes are not
    /// a valid encoding. Schemes that encode no length always succeed.
    fn pad_count(&self, block: &[u8]) -> Result<usize, Self::Error>;
}

#[cfg(test)]
mod tests {
    extern crate std;

    use std::boxed::Box;

    use super::BlockCipherPadding;
    use crate::PaddingError;

    /// 最小的自描述 padding：用固定位元組填滿尾端，再數回尾端連續的該位元組。
    struct TestPadding {
        filler: u8,
    }

    impl BlockCipherPadding for TestPadding {
        type Error = PaddingError;

        fn add_padding(&mut self, block: &mut [u8], position: usize) -> Result<usize, Self::Error> {
            let tail = block
                .get_mut(position..)
                .ok_or(PaddingError::PositionOutOfRange)?;
            tail.fill(self.filler);
            Ok(tail.len())
        }

        fn pad_count(&self, block: &[u8]) -> Result<usize, Self::Error> {
            let count = block
                .iter()
                .rev()
                .take_while(|&&byte| byte == self.filler)
                .count();
            if count == 0 {
                return Err(PaddingError::CorruptPadding);
            }
            Ok(count)
        }
    }

    #[test]
    fn padding_supports_dynamic_dispatch() {
        let mut padding: Box<dyn BlockCipherPadding<Error = PaddingError>> =
            Box::new(TestPadding { filler: 0xa5 });
        let mut block = [0xff_u8; 8];

        assert_eq!(padding.add_padding(&mut block, 5), Ok(3));
        assert_eq!(block, [0xff, 0xff, 0xff, 0xff, 0xff, 0xa5, 0xa5, 0xa5]);
        assert_eq!(padding.pad_count(&block), Ok(3));
    }

    #[test]
    fn a_position_past_the_block_is_rejected() {
        let mut padding = TestPadding { filler: 0xa5 };

        assert_eq!(
            padding.add_padding(&mut [0_u8; 8], 9),
            Err(PaddingError::PositionOutOfRange)
        );
    }

    #[test]
    fn self_describing_schemes_can_report_corruption() {
        let padding = TestPadding { filler: 0xa5 };

        assert_eq!(
            padding.pad_count(&[1, 2, 3, 4]),
            Err(PaddingError::CorruptPadding)
        );
        assert_eq!(padding.pad_count(&[]), Err(PaddingError::CorruptPadding));
    }
}
