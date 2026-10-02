//! Message-authentication-code contracts.

use core::error::Error;

pub trait Mac {
    type Error: Error;
    
    fn mac_size(&self) -> usize;
    
    fn update(&mut self, input: &[u8]) -> Result<(), Self::Error>;
    
    fn do_final(&mut self, output: &mut [u8]) -> Result<usize, Self::Error>;
    
    fn reset(&mut self);
}

pub trait MacInit<P: ?Sized> {
    type Error: Error;
    
    fn init(&mut self, params: &P) -> Result<(), Self::Error>;
}

