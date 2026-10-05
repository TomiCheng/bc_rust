#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Algorithm {
    Aes,
    Aria,
    Des,
    Rc2,
    Rc5,
    Rc5_64,
    Rc6,
}
