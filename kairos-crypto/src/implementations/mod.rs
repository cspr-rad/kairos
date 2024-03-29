mod casper;

#[cfg(feature = "crypto-casper")]
pub type Signer = casper::Signer;

// Alternative crypto implementations can be provided, for example:
//
//#[cfg(feature = "crypto-kairos-a")]
//pub type Signer = kairos_a::Signer;
