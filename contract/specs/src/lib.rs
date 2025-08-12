use templar_common::asset::BorrowAssetAmount;
use templar_market_contract::Contract;

use near_sdk::AccountId;

use cvlr::rule;
use cvlr::cvlr_assert;

#[rule]
pub fn sanity(c: &mut Contract, account_id: AccountId, amount: BorrowAssetAmount) {
    c.execute_supply(account_id, amount);

    cvlr_assert!(false);
}