use crate::IvParams;

pub trait IvOptParams {
    fn iv_opt(&self) -> Option<&[u8]>;
}

impl<T: IvParams + ?Sized> IvOptParams for T {
    fn iv_opt(&self) -> Option<&[u8]> {
        Some(self.iv())
    }
}
