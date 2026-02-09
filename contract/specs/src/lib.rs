#![allow(static_mut_refs)]

use std::borrow::Borrow;

use cvlr::{clog, cvlr_assert, cvlr_assume, cvlr_satisfy, rule};

use near_sdk::AccountId;

use models::templar_nondet::*;

use templar_common::asset::{BorrowAssetAmount, CollateralAssetAmount};
use templar_common::borrow::{BorrowPosition, BorrowPositionGuard, InterestAccumulationProof};
use templar_common::market::Market;
use templar_common::{models, snapshot};
use templar_common::models::split_map::ApplyRule;
use templar_common::oracle::pyth::OracleResponse;
use templar_common::supply::{SupplyPosition, SupplyPositionGuard};
use templar_market_contract::Contract;

#[rule]
pub fn double_borrow_not_allowed() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();

    let amount1 = BorrowAssetAmount::nondet();
    let amount2 = BorrowAssetAmount::nondet();

    cvlr_assume!(!amount1.is_zero());
    cvlr_assume!(!amount2.is_zero());

    let max_amount: u128 = c
        .market
        .configuration
        .borrow_range
        .maximum
        .unwrap()
        .into();

    let total = u128::from(amount1) + u128::from(amount2);
    
    cvlr_assume!(total > max_amount);

    let price_pair = TemplarNondet::nondet();

    // Ensure the borrow position exists.
    let snapshot = c.market.snapshot();
    let mut borrow_position =
        c.market
            .get_or_create_borrow_position_guard(snapshot, account_id.clone());

    // Borrow 1
    let proof_1 = borrow_position.accumulate_interest();
    let first = borrow_position.record_borrow_initial(
        snapshot,
        proof_1,
        amount1,
        &price_pair,
        u64::nondet(),
    );

    cvlr_assume!(first.is_ok());

    // Borrow 2 - should fail
    let proof_2 = borrow_position.accumulate_interest();
    let second = borrow_position.record_borrow_initial(
        snapshot,
        proof_2,
        amount2,
        &price_pair,
        u64::nondet(),
    );

    cvlr_assert!(second.is_err());
}
