use alloc::{collections::BTreeMap, vec, vec::Vec};

use crate::{
    constants::SECURITY_BADGES,
    detail::{get_immediate_caller, get_uref},
    error::DepositError,
};
#[allow(unused_imports)]
use casper_contract::{
    contract_api::{
        self,
        runtime::revert,
        storage::{dictionary_get, dictionary_put},
    },
    ext_ffi,
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_types::{
    bytesrepr::{self, FromBytes, ToBytes},
    CLTyped, Key,
};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SecurityBadge {
    Admin = 0,
}

impl CLTyped for SecurityBadge {
    fn cl_type() -> casper_types::CLType {
        casper_types::CLType::U8
    }
}

impl ToBytes for SecurityBadge {
    fn to_bytes(&self) -> Result<Vec<u8>, bytesrepr::Error> {
        Ok(vec![*self as u8])
    }

    fn serialized_length(&self) -> usize {
        1
    }
}

impl FromBytes for SecurityBadge {
    fn from_bytes(bytes: &[u8]) -> Result<(Self, &[u8]), bytesrepr::Error> {
        Ok((
            match bytes[0] {
                0 => SecurityBadge::Admin,
                _ => return Err(bytesrepr::Error::LeftOverBytes),
            },
            &[],
        ))
    }
}

pub fn access_control_check(allowed_badge_list: Vec<SecurityBadge>) {
    let caller = get_immediate_caller()
        .unwrap_or_revert()
        .to_bytes()
        .unwrap_or_revert();
    if !allowed_badge_list.contains(
        &dictionary_get::<SecurityBadge>(get_uref(SECURITY_BADGES), &base64::encode(caller))
            .unwrap_or_revert()
            .unwrap_or_revert_with(DepositError::InsufficientRights),
    ) {
        revert(DepositError::InsufficientRights)
    }
}

pub fn update_security_badges(badge_map: &BTreeMap<Key, SecurityBadge>) {
    let sec_uref = get_uref(SECURITY_BADGES);
    for (&user, &badge) in badge_map {
        dictionary_put(
            sec_uref,
            &base64::encode(user.to_bytes().unwrap_or_revert()),
            badge,
        )
    }
}
