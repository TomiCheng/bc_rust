use tc_asn1::NamedOid;
use crate::cipher::{Algorithm, Mode, Padding};

#[derive(Clone, Copy, Debug)]
pub struct CipherEntry {
    algo: Algorithm,
    mode: Option<Mode>,
    padding: Option<Padding>,
    name: &'static str,
    oid: Option<NamedOid>,
}
