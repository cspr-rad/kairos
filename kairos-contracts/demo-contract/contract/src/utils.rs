use alloc::vec::Vec;
// Utilities copied from cep-78 and cep-18 implementation.

use casper_contract::contract_api::runtime;
use casper_types::bytesrepr::FromBytes;
use casper_types::{account::AccountHash, system::CallStackElement, Key, U512};

use kairos_crypto::CryptoSigner;

pub mod errors;
use errors::DepositError;

/// Wrap the immediate caller as a Key and return it
fn call_stack_element_to_key(call_stack_element: CallStackElement) -> Key {
    match call_stack_element {
        CallStackElement::Session { account_hash } => Key::from(account_hash),
        CallStackElement::StoredSession { account_hash, .. } => {
            // Stored session code acts in account's context, so if stored session wants to interact
            // with an CEP-18 token caller's address will be used.
            Key::from(account_hash)
        }
        CallStackElement::StoredContract {
            contract_package_hash,
            ..
        } => Key::from(contract_package_hash),
    }
}

/// Traverse the callstack to retrieve the n - 1 th element of the callstack
pub(crate) fn get_immediate_caller() -> Result<Key, DepositError> {
    let call_stack = runtime::get_call_stack();
    call_stack
        .into_iter()
        .rev()
        .nth(1)
        .map(call_stack_element_to_key)
        .ok_or(DepositError::InvalidContext)
}

/// Validates L2 transaction for deposit.
pub(crate) fn validate_deposit_tx(tx_bytes: &[u8], amount: &U512) -> Result<(), DepositError> {
    // Transaction must contain valid deposit body.
    let tx: kairos_tx::asn::Transaction = tx_bytes
        .try_into()
        .map_err(|_e| DepositError::InvalidTransactionData)?;
    let deposit_body = match tx.payload.body {
        kairos_tx::asn::TransactionBody::Deposit(ref body) => Ok(body),
        _ => Err(DepositError::InvalidTransactionType),
    }?;

    // Transaction signer must be the contract caller.
    let tx_raw_public_key: Vec<u8> = tx.public_key.clone().into();
    let (tx_public_key, _rem) = casper_types::PublicKey::from_bytes(&tx_raw_public_key)
        .map_err(|_e| DepositError::FailedToParsePublicKey)?;
    let tx_account_hash: AccountHash = (&tx_public_key).into();
    let caller_account_hash = runtime::get_caller();
    if tx_account_hash != caller_account_hash {
        return Err(DepositError::InvalidTransactionSigner);
    }

    // Transaction amount must be equal to amount transfered to the contract.
    let claimed_amount: u64 = deposit_body
        .amount
        .clone()
        .try_into()
        .map_err(|_e| DepositError::FailedToParseTransactionAmount)?;
    let transfered_amount = match amount.0 {
        [v, 0, 0, 0, 0, 0, 0, 0] => Ok(v),
        _ => Err(DepositError::OverflowTransactionAmount),
    }?;
    if claimed_amount.to_le() != transfered_amount {
        return Err(DepositError::InvalidTransactionAmount);
    }

    // Signature must be valid.
    let signer = kairos_crypto::implementations::Signer::from_public_key(tx_raw_public_key)
        .map_err(|_e| DepositError::FailedToCreateSigner)?;
    signer
        .verify_tx(tx)
        .map_err(|_e| DepositError::InvalidTransactionSignature)?;

    Ok(())
}
