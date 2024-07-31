#![no_std]
#![no_main]
extern crate alloc;
use alloc::vec;
use alloc::{string::ToString, vec::Vec};
use casper_contract::{
    contract_api::{runtime, storage, system},
    unwrap_or_revert::UnwrapOrRevert,
};
use casper_event_standard::Schemas;
use casper_types::bytesrepr::{Bytes, FromBytes, ToBytes};
use casper_types::PublicKey;
use casper_types::{
    contracts::NamedKeys, runtime_args, AccessRights, ApiError, CLValue, EntryPoints, Key,
    RuntimeArgs, URef, U512,
};
use contract_utils::constants::{
    KAIROS_CONTRACT_HASH, KAIROS_CONTRACT_PACKAGE_HASH, KAIROS_CONTRACT_UREF, KAIROS_DEPOSIT_PURSE,
    KAIROS_TRIE_ROOT, KAIROS_UNPROCESSED_DEPOSIT_INDEX, RUNTIME_ARG_AMOUNT,
    RUNTIME_ARG_INITIAL_TRIE_ROOT, RUNTIME_ARG_RECEIPT, RUNTIME_ARG_RECIPIENT,
    RUNTIME_ARG_TEMP_PURSE,
};
mod entry_points;
mod utils;
use kairos_circuit_logic::transactions::{Signed, Withdraw};
use kairos_verifier_risc0_lib::verifier::{Receipt, VerifyError};
use utils::errors::DepositError;
use utils::get_immediate_caller;

#[allow(clippy::single_component_path_imports)]
#[allow(unused)]
use casper_contract_no_std_helpers;

use kairos_circuit_logic::{transactions::L1Deposit, ProofOutputs};

// This entry point is called once when the contract is installed.
// The contract purse will be created in contract context so that it is "owned" by the contract
// rather than the installing account.
#[no_mangle]
pub extern "C" fn init() {
    if runtime::get_key(KAIROS_DEPOSIT_PURSE).is_some() {
        runtime::revert(DepositError::AlreadyInitialized);
    }

    // initialize event schema
    let schemas = Schemas::new().with::<L1Deposit>();
    casper_event_standard::init(schemas);

    let new_deposit_purse: URef = system::create_purse();
    runtime::put_key(KAIROS_DEPOSIT_PURSE, new_deposit_purse.into());
}

#[no_mangle]
pub extern "C" fn get_purse() {
    let deposit_purse: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(DepositError::MissingKeyDepositPurse)
        .into_uref()
        .unwrap_or_revert();
    let reference_to_deposit_purse_with_restricted_access =
        deposit_purse.with_access_rights(AccessRights::ADD);
    runtime::ret(
        CLValue::from_t(reference_to_deposit_purse_with_restricted_access)
            .unwrap_or_revert_with(DepositError::FailedToReturnContractPurseAsReference),
    );
}

// Entry point called by a user through session code to deposit funds.
// Due to Casper < 2.0 purse management and access control, it is necessary that
// a temporary purse is funded and passed to the contract, since this is
// the only secure method of making a payment to a contract purse.
#[no_mangle]
pub extern "C" fn deposit() {
    let temp_purse: URef = runtime::get_named_arg(RUNTIME_ARG_TEMP_PURSE);
    let recipient: casper_types::PublicKey = runtime::get_named_arg(RUNTIME_ARG_RECIPIENT);
    let amount: U512 = runtime::get_named_arg(RUNTIME_ARG_AMOUNT);
    let deposit_purse_uref: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
        .unwrap_or_revert_with(DepositError::MissingKeyDepositPurse)
        .into_uref()
        .unwrap_or_revert_with(ApiError::UnexpectedKeyVariant);
    system::transfer_from_purse_to_purse(temp_purse, deposit_purse_uref, amount, None)
        .unwrap_or_revert();

    // kairos utilizes u64 so only amounts that can be converted are accepted.
    let amount =
        u64::try_from(amount).unwrap_or_else(|_| runtime::revert(ApiError::InvalidArgument));

    // FIXME: verify that the caller's account hash matches a depositor public key argument.
    // We have to ensure we know who the depositor is for regulatory reasons.
    // We could check that the recipient of the funds is the caller or off chain get another signature from the public key.
    let _account_hash = get_immediate_caller().unwrap_or_revert();

    let recipient = recipient.into_bytes().unwrap_or_revert();
    let new_deposit_record: L1Deposit = L1Deposit { recipient, amount };
    // this increases a counter automatically - we don't need to create one ourselves
    casper_event_standard::emit(new_deposit_record);
}

#[no_mangle]
pub extern "C" fn submit_batch() {
    let receipt_serialized: Bytes = runtime::get_named_arg(RUNTIME_ARG_RECEIPT);
    let Ok(receipt): Result<Receipt, _> = serde_json_wasm::from_slice(&receipt_serialized) else {
        runtime::revert(ApiError::User(0u16));
    };

    // In CCTL we hit error_message: "Interpreter error: trap: Code(Unreachable)",
    // In the test execution we finish verification successfully, and hit revert 9999.
    let _ = receipt
        .verify([
            2249819926, 1807275128, 879420467, 753150136, 3885109892, 1252737579, 1362575552,
            43533945,
        ])
        .map_err(|_| runtime::revert(ApiError::User(1u16)));

    runtime::revert(ApiError::User(9999u16));

    let ProofOutputs {
        pre_batch_trie_root,
        post_batch_trie_root,
        deposits,
        withdrawals,
    } = match kairos_verifier_risc0_lib::verifier::verify_execution(&receipt) {
        Ok(proof_outputs) => proof_outputs,
        Err(VerifyError::Ris0ZkvmVerifcationError(_)) => runtime::revert(ApiError::User(1000u16)),
        Err(VerifyError::TooFewBytesInJournal { .. }) => runtime::revert(ApiError::User(1001u16)),
        Err(VerifyError::InvalidLengthInJournal { .. }) => runtime::revert(ApiError::User(1001u16)),
        Err(VerifyError::BorshDeserializationError(_)) => runtime::revert(ApiError::User(1001u16)),
    };

    // get the current root from contract storage
    let trie_root_uref: URef = runtime::get_key(KAIROS_TRIE_ROOT)
        .unwrap_or_revert()
        .into_uref()
        .unwrap_or_revert_with(ApiError::User(2u16));
    let trie_root: Option<[u8; 32]> = storage::read(trie_root_uref)
        .unwrap_or_revert_with(ApiError::User(3u16))
        .unwrap_or_revert_with(ApiError::User(4u16));

    // revert if the previous root of the proof doesn't match the current root
    if trie_root != pre_batch_trie_root {
        runtime::revert(ApiError::User(5u16))
    };

    check_batch_deposits_against_unprocessed(&deposits);
    execute_withdrawals(&withdrawals);

    // store the new root under the contract URef
    storage::write(trie_root_uref, post_batch_trie_root);
    // todo: update sliding window
}

/// Retrive all deposits that have not appeared in a batch yet.
/// Returns the value of `KAIROS_UNPROCESSED_DEPOSIT_INDEX`
/// and an event_index ordered vector of `(event_index, L1Deposit)` tuples.
///
/// This functions error codes are in the range of 101-199.
fn get_unprocessed_deposits() -> (u32, Vec<(u32, L1Deposit)>) {
    let unprocessed_deposits_uref: URef = runtime::get_key(KAIROS_UNPROCESSED_DEPOSIT_INDEX)
        .unwrap_or_revert_with(ApiError::User(101u16))
        .into_uref()
        .unwrap_or_revert_with(ApiError::User(102u16));
    let unprocessed_deposits_index: u32 = storage::read(unprocessed_deposits_uref)
        .unwrap_or_revert_with(ApiError::User(103u16))
        .unwrap_or_revert_with(ApiError::User(104u16));

    let events_length_uref: URef = runtime::get_key(casper_event_standard::EVENTS_LENGTH)
        .unwrap_or_revert_with(ApiError::User(105u16))
        .into_uref()
        .unwrap_or_revert_with(ApiError::User(106u16));
    let events_length: u32 = storage::read(events_length_uref)
        .unwrap_or_revert_with(ApiError::User(107u16))
        .unwrap_or_revert_with(ApiError::User(108u16));

    let events_dict_uref: URef = runtime::get_key(casper_event_standard::EVENTS_DICT)
        .unwrap_or_revert_with(ApiError::User(109u16))
        .into_uref()
        .unwrap_or_revert_with(ApiError::User(110u16));

    let mut unprocessed_deposits: Vec<(u32, L1Deposit)> =
        Vec::with_capacity(events_length as usize);

    for i in unprocessed_deposits_index..events_length {
        match storage::dictionary_get::<Bytes>(events_dict_uref, &i.to_string()) {
            Err(_) => runtime::revert(ApiError::User(111u16)),
            Ok(None) => runtime::revert(ApiError::User(112u16)),
            Ok(Some(event_bytes)) => {
                let (deposit, trailing) = L1Deposit::from_bytes(&event_bytes)
                    .unwrap_or_revert_with(ApiError::User(113u16));

                if !trailing.is_empty() {
                    runtime::revert(ApiError::User(114u16));
                }

                unprocessed_deposits.push((i, deposit));
            }
        };
    }

    (unprocessed_deposits_index, unprocessed_deposits)
}

/// Check that the deposits in the batch match the deposits in the unprocessed deposits list.
/// The batch deposits must an ordered subset of the unprocessed deposits.
///
/// Returns the event index of the first unprocessed deposit that is not present in the batch.
/// If the batch contains all unprocessed deposits,
/// the returned index will point to the next event emitted by the contract.
///
/// Panics: This functions error codes are in the range of 201-299.
fn check_batch_deposits_against_unprocessed(batch_deposits: &[L1Deposit]) -> u32 {
    let (unprocessed_deposits_idx, unprocessed_deposits) = get_unprocessed_deposits();

    // This check ensures that zip does not smuggle fake deposits.
    // Without this check, an attacker could submit a batch with deposits that are not in the unprocessed list.
    if batch_deposits.len() > unprocessed_deposits.len() {
        runtime::revert(ApiError::User(201u16));
    };

    batch_deposits.iter().zip(unprocessed_deposits.iter()).fold(
        unprocessed_deposits_idx,
        |unprocessed_deposits_idx, (batch_deposit, (event_idx, unprocessed_deposit))| {
            if unprocessed_deposits_idx <= *event_idx {
                runtime::revert(ApiError::User(202u16));
            }

            if batch_deposit != unprocessed_deposit {
                runtime::revert(ApiError::User(203u16));
            }
            *event_idx
        },
    )
}

/// Execute the withdrawals from the batch.
/// Errors are in the range of 301-399.
///
/// TODO guard against tiny withdrawals that could be used to spam the contract.
fn execute_withdrawals(withdrawals: &[Signed<Withdraw>]) {
    for withdraw in withdrawals {
        let (recipient, trailing_bytes) = PublicKey::from_bytes(&withdraw.public_key)
            .unwrap_or_revert_with(ApiError::User(301u16));

        if !trailing_bytes.is_empty() {
            runtime::revert(ApiError::User(302u16));
        }

        let amount = U512::from(withdraw.transaction.amount);

        let deposit_purse: URef = runtime::get_key(KAIROS_DEPOSIT_PURSE)
            .unwrap_or_revert_with(ApiError::User(303u16))
            .into_uref()
            .unwrap_or_revert_with(ApiError::User(304u16));

        system::transfer_from_purse_to_account(
            deposit_purse,
            recipient.to_account_hash(),
            amount,
            None,
        )
        .unwrap_or_revert_with(ApiError::User(305u16));
    }
}

#[no_mangle]
pub extern "C" fn call() {
    let entry_points = EntryPoints::from(vec![
        entry_points::init(),
        entry_points::get_purse(),
        entry_points::deposit(),
        entry_points::submit_batch(),
    ]);

    // this counter will be udpated by the entry point that processes / verifies batches
    let last_processed_deposit_counter_uref: URef = storage::new_uref(0u32);

    let initial_trie_root: Option<[u8; 32]> = runtime::get_named_arg(RUNTIME_ARG_INITIAL_TRIE_ROOT);

    let trie_root_uref: URef = storage::new_uref(initial_trie_root);
    let named_keys = NamedKeys::from([
        (
            KAIROS_UNPROCESSED_DEPOSIT_INDEX.to_string(),
            last_processed_deposit_counter_uref.into(),
        ),
        (KAIROS_TRIE_ROOT.to_string(), trie_root_uref.into()),
    ]);

    let (contract_hash, _) = storage::new_locked_contract(
        entry_points,
        Some(named_keys),
        Some(KAIROS_CONTRACT_PACKAGE_HASH.to_string()),
        Some(KAIROS_CONTRACT_UREF.to_string()),
    );

    let contract_hash_key = Key::from(contract_hash);
    runtime::put_key(KAIROS_CONTRACT_HASH, contract_hash_key);

    // Call the init entry point of the newly installed contract
    // This will setup the deposit purse and initialize Event Schemas (CES)
    let init_args = runtime_args! {};
    runtime::call_contract::<()>(contract_hash, "init", init_args);
}
