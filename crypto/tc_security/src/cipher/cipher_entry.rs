use crate::SecurityError;
use crate::cipher::any_params_builder::AnyParamsBuilder;
use crate::cipher::{Algorithm, AnyCipher, AnyParams, Mode, Padding};
use tc_asn1::NamedOid;

#[derive(Clone, Copy, Debug)]
pub struct CipherEntry {
    algo: Algorithm,
    mode: Option<Mode>,
    padding: Option<Padding>,
    name: &'static str,
    oid: Option<NamedOid>,
}

impl CipherEntry {
    pub(super) const fn new(
        algo: Algorithm,
        mode: Option<Mode>,
        padding: Option<Padding>,
        name: &'static str,
        oid: Option<NamedOid>,
    ) -> Self {
        Self {
            algo,
            mode,
            padding,
            name,
            oid,
        }
    }

    pub const fn algorithm(&self) -> Algorithm {
        self.algo
    }

    pub fn iv_size(&self) -> Option<usize> {
        self.mode?.iv_size(self.algo.block_size())
    }

    pub fn builder(&'static self) -> AnyParamsBuilder {
        AnyParamsBuilder::new(self)
    }

    pub fn cipher(&'static self) -> AnyCipher {
        todo!()
    }

    pub(crate) fn build_params(
        &'static self,
        builder: &AnyParamsBuilder,
    ) -> Result<AnyParams, SecurityError> {
        todo!()
    }
}
