//! Implementation of an `Address` which refers either an account hash, or a contract hash.
use alloc::vec::Vec;
use casper_types::{
    account::AccountHash,
    bytesrepr::{self, FromBytes, ToBytes},
    contracts::ContractPackageHash,
    AddressableEntity, AddressableEntityHash, CLType, CLTyped, Key, PackageHash,
};

/// An enum representing an [`AccountHash`] or a [`ContractPackageHash`].
#[derive(PartialOrd, Ord, PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Address {
    /// Represents an account hash.
    Account(AccountHash),
    /// Represents a contract package hash.
    Contract(PackageHash),
}

impl Address {
    /// Returns the inner account hash if `self` is the `Account` variant.
    pub fn as_account_hash(&self) -> Option<&AccountHash> {
        if let Self::Account(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns the inner contract hash if `self` is the `Contract` variant.
    pub fn as_package_hash(&self) -> Option<&PackageHash> {
        if let Self::Contract(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl From<AccountHash> for Address {
    fn from(account_hash: AccountHash) -> Self {
        Self::Account(account_hash)
    }
}

impl From<PackageHash> for Address {
    fn from(package_hash: PackageHash) -> Self {
        Self::Contract(package_hash)
    }
}

impl From<Address> for Key {
    fn from(address: Address) -> Self {
        match address {
            Address::Account(account_hash) => Key::Account(account_hash),
            Address::Contract(contract_package_hash) => Key::Hash(contract_package_hash.value()),
        }
    }
}

impl CLTyped for Address {
    fn cl_type() -> casper_types::CLType {
        CLType::Key
    }
}

/*
impl ToBytes for Address {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        Key::from(*self).to_bytes()
    }

    fn serialized_length(&self) -> usize {
        Key::from(*self).serialized_length()
    }
}

impl FromBytes for Address {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        let (key, remainder) = Key::from_bytes(bytes)?;

        let address = match key {
            Key::Account(account_hash) => Address::Account(account_hash),
            Key::Hash(raw_contract_package_hash) => {
                let contract_package_hash = ContractPackageHash::new(raw_contract_package_hash);
                Address::Contract(contract_package_hash)
            }
            _ => return Err(bytesrepr::Error::Formatting),
        };

        Ok((address, remainder))
    }
}
*/
