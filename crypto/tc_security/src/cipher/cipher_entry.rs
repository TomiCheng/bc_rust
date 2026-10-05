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
    gen_cipher: fn() -> AnyCipher,
    fn_build_params: fn(builder: &AnyParamsBuilder) -> Result<AnyParams, SecurityError>,
}

impl CipherEntry {
    #[allow(dead_code, reason = "表還是空的")]
    pub(super) const fn new(
        algo: Algorithm,
        mode: Option<Mode>,
        padding: Option<Padding>,
        name: &'static str,
        oid: Option<NamedOid>,
        gen_cipher: fn() -> AnyCipher,
        fn_build_params: fn(builder: &AnyParamsBuilder) -> Result<AnyParams, SecurityError>,
    ) -> Self {
        Self {
            algo,
            mode,
            padding,
            name,
            oid,
            gen_cipher,
            fn_build_params,
        }
    }
    pub fn algo(&self) -> Algorithm {
        self.algo
    }
    pub fn mode(&self) -> Option<Mode> {
        self.mode
    }
    pub fn padding(&self) -> Option<Padding> {
        self.padding
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn oid(&self) -> Option<NamedOid> {
        self.oid
    }
    pub fn builder(&'static self) -> AnyParamsBuilder {
        AnyParamsBuilder::new(self)
    }
    pub fn cipher(&self) -> AnyCipher {
        (self.gen_cipher)()
    }
    pub(crate) fn build_params(
        &self,
        builder: &AnyParamsBuilder,
    ) -> Result<AnyParams, SecurityError> {
        (self.fn_build_params)(builder)
    }
}
