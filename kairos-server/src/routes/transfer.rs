use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::instrument;

use kairos_tx::asn::{SigningPayload, TransactionBody};

use crate::routes::PayloadBody;
use crate::state::transactions::{Signed, Transaction, Transfer};
use crate::state::TrieStateThreadMsg;
use crate::{state::LockedBatchState, AppErr};

#[derive(TypedPath)]
#[typed_path("/api/v1/transfer")]
pub struct TransferPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn transfer_handler(
    _: TransferPath,
    State(state): State<LockedBatchState>,
    Json(body): Json<PayloadBody>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");
    let signing_payload: SigningPayload =
        body.payload.as_slice().try_into().context("payload err")?;
    let transfer: Transfer = match signing_payload.body {
        TransactionBody::Transfer(transfer) => transfer.try_into().context("decoding transfer")?,
        _ => {
            return Err(AppErr::set_status(
                anyhow!("invalid transaction type"),
                StatusCode::BAD_REQUEST,
            ))
        }
    };
    let signed = Signed {
        public_key: body.public_key,
        epoch: signing_payload.epoch.try_into().context("decoding epoch")?,
        nonce: signing_payload.nonce.try_into().context("decoding nonce")?,
        transaction: transfer,
    };

    let amount = signed.transaction.amount;
    let from = &signed.public_key;
    let to = &signed.transaction.recipient;

    if amount == 0 {
        return Err(AppErr::set_status(
            anyhow!("transfer amount must be greater than 0"),
            StatusCode::BAD_REQUEST,
        ));
    }

    tracing::info!("TODO: verifying transfer signature");

    // We pre-check this read-only to error early without acquiring the write lock.
    // This prevents a DoS attack exploiting the write lock.
    tracing::info!("verifying transfer sender has sufficient funds");
    check_sender_funds(&state, &signed).await?;

    let mut state = state.write().await;
    let from_balance = state.balances.get_mut(from).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has an account.
                The sender just removed all their funds."
            ),
            StatusCode::CONFLICT,
        )
    })?;

    *from_balance = from_balance.checked_sub(amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has sufficient funds, balance={}, transfer_amount={}.
                The sender just moved their funds in a concurrent request",
                from_balance,
                amount
            ),
            StatusCode::CONFLICT,
        )
    })?;

    let to_balance = state.balances.entry(to.clone()).or_insert_with(|| {
        tracing::info!("creating new account for receiver");
        0
    });

    *to_balance = to_balance.checked_add(amount).ok_or_else(|| {
        AppErr::set_status(anyhow!("Receiver balance overflow"), StatusCode::CONFLICT)
    })?;

    tracing::info!("queuing transaction for trie update");

    let queued_txn = state.queued_transactions.clone();
    // Relase the write lock before queuing the transaction
    drop(state);

    let Signed {
        public_key,
        epoch,
        nonce,
        transaction: transfer,
    } = signed;
    queued_txn
        .send(TrieStateThreadMsg::Transaction(Signed {
            public_key,
            epoch,
            nonce,
            transaction: Transaction::Transfer(transfer),
        }))
        .await
        .context("sending transaction to trie thread")?;

    Ok(())
}

async fn check_sender_funds(
    state: &LockedBatchState,
    Signed {
        public_key,
        epoch,
        transaction: Transfer { recipient, amount },
        ..
    }: &Signed<Transfer>,
) -> Result<(), AppErr> {
    let state = state.read().await;
    if *epoch != state.batch_epoch {
        return Err(AppErr::set_status(
            anyhow!("epoch mismatch"),
            StatusCode::CONFLICT,
        ));
    }

    let from_balance = state.balances.get(public_key).ok_or_else(|| {
        AppErr::set_status(
            anyhow!("Sender does not have an account"),
            StatusCode::BAD_REQUEST,
        )
    })?;

    from_balance.checked_sub(*amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender does not have sufficient funds, balance={}, transfer_amount={}",
                from_balance,
                amount
            ),
            StatusCode::FORBIDDEN,
        )
    })?;

    let to_balance = state.balances.get(recipient).unwrap_or(&0);
    if to_balance.checked_add(*amount).is_none() {
        return Err(AppErr::set_status(
            anyhow!("Receiver balance overflow"),
            StatusCode::CONFLICT,
        ));
    }

    Ok(())
}
