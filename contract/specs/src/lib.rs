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

// #[rule]
pub fn split_ok_1() {
    let i: u32 = TemplarNondet::nondet();
    let j: u32 = TemplarNondet::nondet();
    let v: u32 = TemplarNondet::nondet();

    let mut m: models::hash_map::HashMap<u32, u32> = models::hash_map::HashMap::nondet();
    m.apply_rule(i, None);
    let _ = m.entry(i).or_insert(v);
    let v2 = m.get(&j);
    v2.map(|the_val| cvlr_satisfy!(*the_val == v));
    v2.map(|the_val| cvlr_satisfy!(*the_val != v));
}

// #[rule]
pub fn accounts_can_be_neq() {
    // let a1: AccountId = AccountId::nondet();
    // let a2: AccountId = a1.clone();
    let s = AccountId::nondet();
    let t = AccountId::nondet();
    cvlr_satisfy!(s != t);
}

#[rule]
pub fn add_incoming_sanity() {
    let account_id = AccountId::nondet();
    let mut market = Market::nondet();
    let position = SupplyPosition::nondet();
    let mut supply_pos_guard = SupplyPositionGuard::new(&mut market, account_id, position);
    let amount = TemplarNondet::nondet();
    let block_ts = u64::nondet();
    let proof = supply_pos_guard.accumulate_yield();
    supply_pos_guard.record_deposit(proof, amount, block_ts);
    cvlr_satisfy!(true);
}

#[rule]
pub fn snapshot_sanity() {
    let mut market = Market::nondet();
    market.snapshot();
    cvlr_satisfy!(true);
}

#[rule]
pub fn accumulate_interest_sanity() {
    let mut market = Market::nondet();
    let account_id = AccountId::nondet();
    let borrow_position = BorrowPosition::nondet();
    let mut bp_guard = BorrowPositionGuard::new(&mut market, account_id, borrow_position.clone());
    bp_guard.accumulate_interest();
    cvlr_satisfy!(true);
}

#[rule]
pub fn record_borrow_asset_protocol_yield_integrity() {
    let amount = TemplarNondet::nondet(); //wrap to type BorrowAssetAmount
    let mut market = Market::nondet(); //nondet Market, mutable
    let protocol_id = market.configuration.protocol_account_id.clone();

    market.static_yield.focus(protocol_id.clone());

    let yield_borrow_asset_protocol_pre = market
        .static_yield
        .get(&protocol_id)
        .unwrap_or_default()
        .borrow_asset;

    market.record_borrow_asset_protocol_yield(amount);

    let yield_borrow_asset_protocol_post = market
        .static_yield
        .get(&protocol_id)
        .unwrap_or_default()
        .borrow_asset;
    cvlr_assert!(
        u128::from(yield_borrow_asset_protocol_post)
            == u128::from(yield_borrow_asset_protocol_pre) + u128::from(amount)
    );
}

#[rule]
pub fn record_borrow_asset_yield_distribution_integrity_1() {
    let amount = BorrowAssetAmount::nondet();
    let mut market = Market::nondet();

    let account_id = AccountId::nondet();

    market.static_yield.focus(account_id.clone());

    let static_yield_account_before = market.static_yield.get(&account_id);

    market.record_borrow_asset_yield_distribution(amount);

    let static_yield_acount_after = market.static_yield.get(&account_id);

    cvlr_assert!(static_yield_acount_after >= static_yield_account_before);
}

#[rule]
pub fn record_borrow_asset_yield_distribution_integrity_2() {
    let amount = BorrowAssetAmount::nondet();
    let mut market = Market::nondet();

    let account_id = AccountId::nondet();

    market.static_yield.focus(account_id.clone());

    let yield_weight_account = market
        .configuration
        .yield_weights
        .r#static
        .get(&account_id)
        .copied();

    let static_yield_account_before = market.static_yield.get(&account_id);

    market.record_borrow_asset_yield_distribution(amount);

    let static_yield_acount_after = market.static_yield.get(&account_id);

    cvlr_assert!(
        !(static_yield_acount_after > static_yield_account_before)
            || yield_weight_account > Some(0)
    );
}

#[rule]
pub fn accumulate_interest_integrity_1() {
    let borrow_position = BorrowPosition::nondet();

    let fees_pre = borrow_position.borrow_asset_fees;
    let fees_pre_total = fees_pre.get_total();

    let mut market = Market::nondet();
    let account_id = AccountId::nondet();

    let mut bp_guard = BorrowPositionGuard::new(&mut market, account_id, borrow_position);
    bp_guard.accumulate_interest();

    let borrow_position = bp_guard.inner();
    let fees_post = borrow_position.borrow_asset_fees;

    let fees_post_total = fees_post.get_total();
    cvlr_assert!(fees_post_total >= fees_pre_total);
}

#[rule]
pub fn snapshot_with_yield_distribution_integrity() {
    let amount = BorrowAssetAmount::nondet();
    let mut market = Market::nondet();

    let current_snapshot_before = &market.current_snapshot.clone();
    let mut snapshot_yield_distribution_before = current_snapshot_before.yield_distribution();

    market.snapshot_with_yield_distribution(amount);

    let current_snapshot_after = &market.current_snapshot;
    let snapshot_yield_distribution_after = current_snapshot_after.yield_distribution();

    let time_chunk_changed =
        current_snapshot_after.time_chunk() != current_snapshot_before.time_chunk();

    if time_chunk_changed {
        cvlr_assert!(snapshot_yield_distribution_after == amount);
    } else {
        let _ = snapshot_yield_distribution_before.join(amount);
        cvlr_assert!(snapshot_yield_distribution_after == snapshot_yield_distribution_before);
    }
}

#[rule]
pub fn double_borrow_fails() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let amount1 = BorrowAssetAmount::nondet();
    let amount2 = BorrowAssetAmount::nondet();
    let max_amount = c
        .market
        .configuration
        .borrow_range
        .maximum
        .unwrap()
        .amount
        .0;
    clog!(amount1.amount.0);
    clog!(amount2.amount.0);
    clog!(max_amount);
    c.compute_amount(amount1);
    c.borrow_01_consume_price_internal(account_id, amount1, oracle);
    c.compute_amount(amount2);
    cvlr_assert!(amount1.amount.0 + amount2.amount.0 <= max_amount);
}

#[rule]
pub fn execute_supply_sanity() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    let amount = BorrowAssetAmount::nondet();
    c.execute_supply(account_id, amount);
    cvlr_assert!(false);
}

#[rule]
pub fn borrow_preserves_health() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    c.market.focus_borrow_positions(account_id.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());
    let amount = TemplarNondet::nondet();
    c.borrow_01_consume_price_internal(account_id.clone(), amount, oracle);
    {
        let mut borrow_position = c.borrow_position_guard(account_id.clone()).unwrap();
        let ok2 = borrow_position.satisfies_mcr_maintenance(&price);
        cvlr_assert!(ok2);
    }
}

#[rule]
pub fn withdraw_preserves_health() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    c.market.focus_borrow_positions(account_id.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());
    let amount = TemplarNondet::nondet();
    c.withdraw_collateral_01_consume_price_internal(account_id.clone(), amount, oracle);
    {
        let mut borrow_position = c.borrow_position_guard(account_id.clone()).unwrap();
        let ok2 = borrow_position.satisfies_mcr_maintenance(&price);
        cvlr_assert!(ok2);
    }
}

#[rule]
pub fn collateralize_preserves_liquidation_state() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    c.market.focus_borrow_positions(account_id.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());
    let amount = TemplarNondet::nondet();

    c.execute_collateralize(account_id.clone(), amount, &price);
    {
        let mut borrow_position = c.borrow_position_guard(account_id.clone()).unwrap();
        let ok2 = borrow_position.is_eligible_for_liquidation(&price, u64::nondet());
        cvlr_assert!(!ok2);
    }
}

#[rule]
pub fn collateralize_preserves_liquidation_state_sanity() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    c.market.focus_borrow_positions(account_id.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());
    let amount = TemplarNondet::nondet();

    c.execute_collateralize(account_id.clone(), amount, &price);
    let mut borrow_position = c.borrow_position_guard(account_id.clone()).unwrap();
    borrow_position.is_eligible_for_liquidation(&price, u64::nondet());
    cvlr_satisfy!(true);
}

#[rule]
pub fn withdraw_preserves_health_sanity() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    c.market.focus_borrow_positions(account_id.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());
    let amount = TemplarNondet::nondet();
    c.withdraw_collateral_01_consume_price_internal(account_id.clone(), amount, oracle);
    {
        let mut borrow_position = c.borrow_position_guard(account_id.clone()).unwrap();
        borrow_position.satisfies_mcr_maintenance(&price);
    }
    cvlr_satisfy!(true);
}

#[rule]
pub fn borrow_preserves_health_sanity() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    c.market.focus_borrow_positions(account_id.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());
    let amount = TemplarNondet::nondet();
    c.borrow_01_consume_price_internal(account_id.clone(), amount, oracle);
    {
        let mut borrow_position = c.borrow_position_guard(account_id.clone()).unwrap();
        borrow_position.satisfies_mcr_maintenance(&price);
    }
    cvlr_satisfy!(true);
}

#[rule]
pub fn collateralize_collateral_increase_monotonicity() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    c.market.focus_borrow_positions(account_id.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());
    let amount = TemplarNondet::nondet();

    let collateral_before = c
        .borrow_position_ref(account_id.clone())
        .unwrap()
        .inner()
        .collateral_asset_deposit
        .amount
        .0;

    clog!(collateral_before);

    c.execute_collateralize(account_id.clone(), amount, &price);

    let collateral_after = c
        .borrow_position_ref(account_id.clone())
        .unwrap()
        .inner()
        .collateral_asset_deposit
        .amount
        .0;

    clog!(amount.amount.0);
    clog!(collateral_after);

    cvlr_assert!(collateral_after == collateral_before + amount.amount.0);
}

#[rule]
pub fn collateralize_interest_accumulation_consistency() {
    let mut c = Contract::nondet();
    let account_id = AccountId::nondet();
    c.market.focus_borrow_positions(account_id.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());
    let amount: CollateralAssetAmount = TemplarNondet::nondet();

    let fees_before = c
        .borrow_position_ref(account_id.clone())
        .unwrap()
        .position
        .borrow_asset_fees
        .get_total()
        .amount
        .0;

    clog!(amount.amount.0);
    clog!(fees_before);

    c.execute_collateralize(account_id.clone(), amount, &price);

    let fees_after = c
        .borrow_position_ref(account_id.clone())
        .unwrap()
        .position
        .borrow_asset_fees
        .get_total()
        .amount
        .0;
    clog!(fees_after);

    cvlr_assert!(fees_after >= fees_before);
}

#[rule]
pub fn repay_liability_reduction_reverts_correctly() {
    let mut p = BorrowPosition::nondet();
    cvlr_assume!(p.is_liquidation_locked);
    let p_pre = p.clone();
    let amount = BorrowAssetAmount::nondet();
    let min = BorrowAssetAmount::nondet();
    let r = p.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), amount, min);
    cvlr_assert!(r.is_err());
    cvlr_assert!(p == p_pre);
}

#[rule]
pub fn repay_amount_to_fees_value() {
    let mut p = BorrowPosition::nondet();
    let fees_pre = p.borrow_asset_fees.get_total();
    let amount = BorrowAssetAmount::nondet();
    let min = BorrowAssetAmount::nondet();
    let r = p
        .reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), amount, min)
        .unwrap();
    let fees_post = p.borrow_asset_fees.get_total();
    cvlr_assert!(fees_post.amount.0 + r.amount_to_fees.amount.0 == fees_pre.amount.0);
    cvlr_assert!(fees_post <= fees_pre);
}

#[rule]
fn repay_principal_decreases_exactly() {
    let mut p = BorrowPosition::nondet();
    let principal_pre = p.borrow_asset_principal;
    let amount = BorrowAssetAmount::nondet();
    let min = BorrowAssetAmount::nondet();
    let r = p.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), amount, min).unwrap();
    let principal_post = p.borrow_asset_principal;
    cvlr_assert!(principal_pre.amount.0 == principal_post.amount.0 + r.amount_to_principal.amount.0);
    cvlr_assert!(principal_post <= principal_pre);
}


#[rule]
fn repay_zero_principal_behavior() {
    let mut p = BorrowPosition::nondet();
    cvlr_assume!(p.borrow_asset_principal.is_zero());
    let amount = BorrowAssetAmount::nondet();
    let min = BorrowAssetAmount::nondet();
    let lr = p.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), amount, min).unwrap();
    cvlr_assert!(lr.amount_to_principal.is_zero());
}

#[rule]
fn repay_components_bounded() {
    let mut p = BorrowPosition::nondet();
    let fees_pre = p.borrow_asset_fees.get_total();
    let princ_pre = p.borrow_asset_principal;
    let amount = BorrowAssetAmount::nondet();
    let min = BorrowAssetAmount::nondet();
    let lr = p.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), amount, min).unwrap();

    cvlr_assert!(lr.amount_to_fees <= fees_pre);
    cvlr_assert!(lr.amount_to_principal <= princ_pre);
    cvlr_assert!(lr.amount_refund <= amount);
}

#[rule]
pub fn repay_zero_payment_noop() {
    let mut p = BorrowPosition::nondet();
    let fees_pre = p.borrow_asset_fees.get_total();
    let princ_pre = p.borrow_asset_principal;
    let min = BorrowAssetAmount::nondet();
    let lr = p.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), BorrowAssetAmount::from(0u128), min).unwrap();
    cvlr_assert!(p.borrow_asset_fees.get_total() == fees_pre);
    cvlr_assert!(p.borrow_asset_principal == princ_pre);
    cvlr_assert!(lr.amount_refund.is_zero());
}

#[rule]
pub fn repay_timestamp_clear_when_fully_paid() {
    let mut p = BorrowPosition::nondet();
    let amount = BorrowAssetAmount::nondet();
    let min = BorrowAssetAmount::nondet();
    p.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), amount, min).unwrap();
    cvlr_assert!(!p.borrow_asset_principal.is_zero() || p.started_at_block_timestamp_ms.is_none());
}

#[rule]
pub fn repay_split_payments_combine() {
    let mut p1 = BorrowPosition::nondet();
    let mut p2 = p1.clone();

    let min = BorrowAssetAmount::nondet();
    let x = BorrowAssetAmount::nondet();
    let y = BorrowAssetAmount::nondet();

    let _ = p1.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), x, min);
    let _ = p1.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), y, min);

    let _ = p2.reduce_borrow_asset_liability(InterestAccumulationProof::nondet(), (x.amount.0 + y.amount.0).into(), min);

    cvlr_assert!(p1.borrow_asset_fees.get_total() == p2.borrow_asset_fees.get_total());
    cvlr_assert!(p1.borrow_asset_principal >= p2.borrow_asset_principal);
}


#[rule]
pub fn liquidate_initial_sets_lock() {
    let mut c = Contract::nondet();
    let account = AccountId::nondet();

    c.focus_borrow_positions(account.clone());

    let oracle = OracleResponse {
        asset1: c
            .configuration
            .price_oracle_configuration
            .borrow_asset_price_id,
        price1: Some(TemplarNondet::nondet()),
        asset2: c
            .configuration
            .price_oracle_configuration
            .collateral_asset_price_id,
        price2: Some(TemplarNondet::nondet()),
        bot: None.into(),
    };
    let price = c.price_pair(oracle.clone());

    let amount = BorrowAssetAmount::nondet();

    let _ = c.execute_liquidate_initial(account.clone(), amount, &price);
    let bp = c.borrow_position_guard(account).unwrap().inner().clone();
    cvlr_assert!(bp.is_liquidation_locked);
}

#[rule]
pub fn liquidation_final_success_clears() {
    let mut c = Contract::nondet();
    let account = AccountId::nondet();
    let liquidator  = AccountId::nondet();
    c.focus_borrow_positions(account.clone());

    let amount = BorrowAssetAmount::nondet();

    let refund = c.execute_liquidate_final(liquidator, account.clone(), amount, true);
    cvlr_assert!(refund.is_zero());
}

#[rule]
pub fn liquidation_final_fail_unlocks() {
    let mut c = Contract::nondet();
    let account = AccountId::nondet();
    let liquidator  = AccountId::nondet();
    c.focus_borrow_positions(account.clone());

    let amount = BorrowAssetAmount::nondet();

    let refund = c.execute_liquidate_final(liquidator, account.clone(), amount, false);
    let bp = c.borrow_position_guard(account).unwrap().inner().clone();
    cvlr_assert!(refund.amount.0 == amount.amount.0);
    cvlr_assert!(bp.is_liquidation_locked == false);
}

#[rule]
pub fn full_liquidation_updates_correctly() {
    let mut c = Contract::nondet();
    let account = AccountId::nondet();
    let liquidator = AccountId::nondet();
    c.focus_borrow_positions(account.clone());

    let mut bp = c.borrow_position_guard(account).unwrap();
    let principal = bp.inner().get_borrow_asset_principal().amount.0;
    
    let borrow_asset_borrowed_before = bp.market.borrow_asset_borrowed.amount.0;
    
    let amount = BorrowAssetAmount::nondet();
    
    bp.record_full_liquidation(liquidator, amount);
    
    let borrow_asset_borrowed_after = bp.market.borrow_asset_borrowed.amount.0;
    
    cvlr_assert!(!bp.inner().is_liquidation_locked);
    cvlr_assert!(bp.inner().started_at_block_timestamp_ms.is_none());
    cvlr_assert!(borrow_asset_borrowed_after == borrow_asset_borrowed_before - principal);
}