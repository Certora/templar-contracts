use cvlr::{clog, cvlr_assert, cvlr_satisfy, nondet, rule};

use near_sdk::{ AccountId};

use models::templar_nondet::*;

use templar_common::asset::BorrowAssetAmount;
use templar_common::borrow::{BorrowPosition, BorrowPositionGuard};
use templar_common::market::{Market, WithdrawalResolution};
use templar_common::models::split_map::ApplyRule;
use templar_common::oracle::pyth::OracleResponse;
use templar_common::supply::{SupplyPosition, SupplyPositionGuard};
use templar_common::{models};
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
pub fn record_borrow_asset_protocol_yield_intergity() {
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
pub fn withdraws_decrease_available_correctly() {
    let market = Market::nondet();
    let mut c = Contract::nondet();
	let available_pre = market.get_borrow_asset_available_to_borrow().amount.0;
	
    let withdrawal_resolution = WithdrawalResolution::nondet();
    let expected_success: bool = nondet();
	let withdrawl_request = c.withdrawal_queue.try_pop().unwrap();
	let amount_withdrawn = withdrawl_request.1.amount.0;
	
	c.execute_next_supply_withdrawal_request_01_finalize(withdrawal_resolution, expected_success);
	
	let available_post = market.get_borrow_asset_available_to_borrow().amount.0;
	cvlr_assert!(available_pre == available_post - amount_withdrawn);
}

// timing out, needs investigation
#[rule]
pub fn double_borrow_fails() {
    let market = Market::nondet();
    let c = Contract::nondet();
    let max_amount = market.configuration.borrow_range.maximum.unwrap();
    c.compute_amount(max_amount);
    c.compute_amount(max_amount);
    cvlr_assert!(false);
}

#[rule]
pub fn snapshot_with_yield_and_supplier_position() {
    let mut market = Market::nondet();
    let borrow_amount = BorrowAssetAmount::nondet();
    let amount = borrow_amount.amount;
    let account = AccountId::nondet();
    let yield_pre;
    let mut position = SupplyPosition::nondet();
    {
        let mut sp_guard = SupplyPositionGuard::new(&mut market, account.clone(), position);

        sp_guard.accumulate_yield(); // after this the yield is up-to-date

        position = sp_guard.inner().clone();
        yield_pre = position.borrow_asset_yield.total;
    }

    market.snapshot_with_yield_distribution(borrow_amount);

    let mut sp_guard = SupplyPositionGuard::new(&mut market, account, position);
    sp_guard.accumulate_yield(); // after this the yield is up-to-date

    {
        let position = sp_guard.inner();
        let yield_post = position.borrow_asset_yield.total;
        clog!(yield_post.amount.0);
        clog!(yield_pre.amount.0);
        clog!(amount.0);
        cvlr_assert!(yield_post.amount.0 <= yield_pre.amount.0 + amount.0);
    }
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