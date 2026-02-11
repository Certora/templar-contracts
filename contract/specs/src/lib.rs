#![allow(static_mut_refs)]

use cvlr::{cvlr_assert, cvlr_assume, rule};

mod borrow;

use crate::borrow::{SimpleBorrowPositionGuard, SimpleMarket, SimpleMarketConfiguration};
use templar_common::asset::BorrowAssetAmount;
use templar_common::borrow::BorrowPosition;
use templar_common::market::ValidAmountRange;
use templar_common::models::templar_nondet::TemplarNondet;
use templar_common::number::Decimal;

// https://prover.certora.com/output/33158/46ee9bf4424c48e387614c7637c236f3
#[rule]
pub fn double_borrow_not_allowed() {

    let amount1 = BorrowAssetAmount::nondet();
    let amount2 = BorrowAssetAmount::nondet();

    let minimum = BorrowAssetAmount::nondet();
    let maximum = Some(BorrowAssetAmount::nondet());

    let borrow_range = ValidAmountRange::try_from((minimum, maximum)).unwrap();
    let max = borrow_range.maximum.unwrap();

    let mut market = SimpleMarket {
        configuration: SimpleMarketConfiguration {
            borrow_range,
            borrow_asset_maximum_usage_ratio: TemplarNondet::nondet(),
            borrow_origination_fee: TemplarNondet::nondet(),
        },
        borrow_asset_balance: TemplarNondet::nondet(),
        borrow_asset_deposited_active: TemplarNondet::nondet(),
        borrow_asset_deposited_incoming_total: TemplarNondet::nondet(),
        borrow_asset_borrowed_in_flight: TemplarNondet::nondet(),
        current_yield_distribution: TemplarNondet::nondet(),
        single_snapshot_maximum_interest_precomputed: TemplarNondet::nondet(),
    };

    let position: BorrowPosition = TemplarNondet::nondet();

    let mut bpg = SimpleBorrowPositionGuard::new(&mut market, position);

    let first = bpg.record_borrow_initial(amount1);
    cvlr_assume!(first.is_ok());


    let current_principal = u128::from(bpg.borrow_asset_principal());
    cvlr_assume!(current_principal <= u128::MAX - u128::from(amount2));

    let total_after_second = BorrowAssetAmount::from(current_principal + u128::from(amount2));
    cvlr_assume!(total_after_second > max);

    let second = bpg.record_borrow_initial(amount2);

    cvlr_assert!(second.is_err());
}
