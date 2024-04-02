use anyhow::{anyhow, Context};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::routing::TypedPath;
use tracing::*;

use kairos_tx::asn::{SigningPayload, TransactionBody};

use crate::routes::PayloadBody;
use crate::state::transactions::{Signed, Transaction, Withdraw};
use crate::state::TrieStateThreadMsg;
use crate::{state::LockedBatchState, AppErr, PublicKey};

#[derive(Debug, TypedPath)]
#[typed_path("/api/v1/withdraw")]
pub struct WithdrawPath;

#[instrument(level = "trace", skip(state), ret)]
pub async fn withdraw_handler(
    _: WithdrawPath,
    State(state): State<LockedBatchState>,
    Json(body): Json<PayloadBody>,
) -> Result<(), AppErr> {
    tracing::info!("parsing transaction data");
    let signing_payload: SigningPayload =
        body.payload.as_slice().try_into().context("payload err")?;
    let withdrawal = match signing_payload.body {
        TransactionBody::Withdrawal(withdrawal) => withdrawal,
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
        transaction: Withdraw::try_from(withdrawal).context("decoding withdrawal")?,
    };
    let amount = signed.transaction.amount;
    let public_key = &signed.public_key;

    tracing::info!("TODO: verifying withdrawal signature");

    tracing::info!("verifying withdrawal sender has sufficient funds");
    check_sender_funds(&state, public_key, amount, signed.epoch).await?;

    tracing::info!("TODO: adding withdrawal to batch");

    let mut state = state.write().await;
    let from_balance = state.balances.get_mut(public_key).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has an account.
                The sender just removed all their funds."
            ),
            StatusCode::CONFLICT,
        )
    })?;

    let updated_balance = from_balance.checked_sub(amount).ok_or_else(|| {
        AppErr::set_status(
            anyhow!(
                "Sender no longer has sufficient funds, balance={}, withdrawal_amount={}.
                The sender just moved their funds in a concurrent request",
                from_balance,
                amount
            ),
            StatusCode::CONFLICT,
        )
    })?;

    *from_balance = updated_balance;

    if updated_balance == 0 {
        state.balances.remove(public_key);
    }

    tracing::info!(
        "Updated account public_key={:?} balance={}",
        public_key,
        updated_balance
    );

    tracing::info!("queuing withdrawal transaction");

    let queued_txn = state.queued_transactions.clone();
    // Relase the write lock before queuing the transaction
    drop(state);

    queued_txn
        .send(TrieStateThreadMsg::Transaction(Signed {
            public_key: public_key.clone(),
            epoch: signed.epoch,
            nonce: signed.nonce,
            transaction: Transaction::Withdraw(signed.transaction),
        }))
        .await
        .context("sending transaction to trie thread")?;

    Ok(())
}

async fn check_sender_funds(
    state: &LockedBatchState,
    public_key: &PublicKey,
    amount: u64,
    epoch: u64,
) -> Result<(), AppErr> {
    if amount == 0 {
        return Err(AppErr::set_status(
            anyhow!("withdrawal amount must be greater than 0"),
            StatusCode::BAD_REQUEST,
        ));
    }

    let state = state.read().await;

    if state.batch_epoch != epoch {
        return Err(AppErr::set_status(
            anyhow!("Deposit epoch does not match current batch epoch."),
            StatusCode::CONFLICT,
        ));
    }

    let from_balance = state.balances.get(public_key).ok_or_else(|| {
        AppErr::set_status(anyhow!("Withdrawer has no account."), StatusCode::CONFLICT)
    })?;

    if *from_balance < amount {
        return Err(AppErr::set_status(
            anyhow!(
                "Withdrawer has insufficient funds, balance={}, withdrawal_amount={}.",
                from_balance,
                amount
            ),
            StatusCode::FORBIDDEN,
        ));
    }

    Ok(())
}
