//! Buffering logic shared by the allocating and fixed-size block adapters.

use tc_block_modes::BlockCipherMode;

use crate::BufferedError;

pub(super) fn update_output_len<E>(
    buffered: usize,
    block_size: usize,
    input_len: usize,
) -> Result<usize, BufferedError<E>> {
    let total = buffered
        .checked_add(input_len)
        .ok_or(BufferedError::InputTooLong)?;
    Ok(total - total % block_size)
}

pub(super) fn output_len<E>(buffered: usize, input_len: usize) -> Result<usize, BufferedError<E>> {
    buffered
        .checked_add(input_len)
        .ok_or(BufferedError::InputTooLong)
}

pub(super) fn process_bytes<C: BlockCipherMode>(
    cipher_mode: &mut C,
    buffer: &mut [u8],
    buffered: &mut usize,
    mut input: &[u8],
    output: &mut [u8],
) -> Result<usize, BufferedError<C::Error>> {
    if input.is_empty() {
        return Ok(0);
    }

    let block_size = buffer.len();
    let required = update_output_len(*buffered, block_size, input.len())?;
    if output.len() < required {
        return Err(BufferedError::OutputTooShort {
            required,
            available: output.len(),
        });
    }

    let available = block_size - *buffered;
    let mut written = 0;

    if input.len() >= available {
        buffer[*buffered..].copy_from_slice(&input[..available]);
        input = &input[available..];
        *buffered = 0;

        written += cipher_mode
            .process_block(buffer, &mut output[written..])
            .map_err(BufferedError::Cipher)?;

        while input.len() >= block_size {
            written += cipher_mode
                .process_block(input, &mut output[written..])
                .map_err(BufferedError::Cipher)?;
            input = &input[block_size..];
        }
    }

    let end = *buffered + input.len();
    buffer[*buffered..end].copy_from_slice(input);
    *buffered = end;
    Ok(written)
}

/// Resolves the trailing partial block through `scratch`, which must be as
/// long as `buffer`; the caller wipes both afterwards.
pub(super) fn do_final<C: BlockCipherMode>(
    cipher_mode: &mut C,
    buffer: &mut [u8],
    scratch: &mut [u8],
    buffered: usize,
    output: &mut [u8],
) -> Result<usize, BufferedError<C::Error>> {
    if buffered == 0 {
        return Ok(0);
    }
    if !cipher_mode.is_partial_block_okay() {
        return Err(BufferedError::IncompleteLastBlock);
    }
    if output.len() < buffered {
        return Err(BufferedError::OutputTooShort {
            required: buffered,
            available: output.len(),
        });
    }

    buffer[buffered..].fill(0);
    cipher_mode
        .process_block(buffer, scratch)
        .map_err(BufferedError::Cipher)?;
    output[..buffered].copy_from_slice(&scratch[..buffered]);
    Ok(buffered)
}
