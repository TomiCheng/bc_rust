mod unit;

#[cfg(feature = "alloc")]
pub(crate) use unit::decode_vec;
pub(crate) use unit::{
    Unit, decode_into, fits, input_fill, sign_fill, units_for_bits, write_be, write_le,
};
