use models::templar_nondet::*;
use cvlr::{cvlr_assert, cvlr_assume};
use cvlr::{cvlr_satisfy, rule};
use near_sdk::AccountId;

use templar_common::asset::BorrowAssetAmount;
use templar_common::borrow::{BorrowPosition, BorrowPositionGuard, InterestAccumulationProof};
use templar_common::market::Market;
use templar_common::models::split_map::ApplyRule;
use templar_common::supply::{SupplyPositionGuard};
use templar_common::{models};


#[rule]
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

#[no_mangle]
#[inline(never)]
pub fn foo() -> Box<str> {
    unsafe {
        let bytes = CERTORA_nondet_bytes(2);
        String::from_raw_parts(bytes, 2, 2).into_boxed_str()
    }
}
#[rule]
pub fn accounts_can_be_neq() {
    // let a1: AccountId = AccountId::nondet();
    // let a2: AccountId = a1.clone();
    let s = AccountId::nondet();
    let t = AccountId::nondet();
    cvlr_satisfy!(s != t);
}


#[inline(never)]
pub fn id<T>(t: T) -> T { t }

// #[no_mangle]
// pub fn unsafe_box_str_clone(b: &Box<str>) -> Box<str> {
//     unsafe {
//         let p = b.as_ptr();
//         let bytes = CERTORA_nondet_bytes(b.len() as u32);
//         std::ptr::copy(p, bytes, b.len());
//         let new_s = nondet_bytes_sz(b.len());
//         id(new_s.into_boxed_str())
//     }
// }

#[no_mangle]
#[inline(never)]
pub fn unsafe_account_id_clone(a: &AccountId) -> AccountId {
    unsafe {
        let ai: u64 = std::mem::transmute(a.as_bytes());
        std::mem::transmute(ai)
    }
}

#[no_mangle]
#[inline(never)]
pub fn unsafe_account_id_eq(a: &AccountId, b: &AccountId) -> bool {
    unsafe {
        let ai: u64 = std::mem::transmute(a.as_bytes());
        let bi: u64 = std::mem::transmute(b.as_bytes());
        ai == bi
    }
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
pub fn add_incoming_sanity(mut supply_pos_guard: SupplyPositionGuard) {
    let amount = TemplarNondet::nondet();
    let block_ts = u64::nondet();
    let proof = supply_pos_guard.accumulate_yield();
    supply_pos_guard.record_deposit(proof, amount, block_ts);
    cvlr_assert!(false);
}

#[rule]
pub fn snapshot_sanity() {
    let mut market = Market::nondet();
    market.snapshot();
    cvlr_assert!(false);
}

#[rule]
pub fn accumulate_interest_sanity() {
    let mut market = Market::nondet();
    let account_id = AccountId::nondet();
    let borrow_position = BorrowPosition::nondet();
    let mut bp_guard = BorrowPositionGuard::new(&mut market, account_id, borrow_position.clone());
    bp_guard.accumulate_interest();
    cvlr_assert!(false);
}

#[rule]
pub fn borrow_preserves_health() {
    let mut market = Market::nondet();
    let account_id = AccountId::nondet();
    let amount = BorrowAssetAmount::nondet();
    let fees = BorrowAssetAmount::nondet();
    let borrow_position = BorrowPosition::nondet();
    let price_pair = TemplarNondet::nondet();
    let block_ts = u64::nondet();

    let heath_pre = market
        .configuration
        .borrow_status(&borrow_position, &price_pair, block_ts);

    {
        let mut bp_guard =
            BorrowPositionGuard::new(&mut market, account_id, borrow_position.clone());

        let proof = InterestAccumulationProof::nondet();
        cvlr_assume!(heath_pre.is_healthy());

        bp_guard.record_borrow_asset_withdrawal(proof, amount, fees);
    }

    // let heath_post = market
    //     .configuration
    //     .borrow_status(&borrow_position, &price_pair, block_ts);

    //  cvlr_assert!(heath_post.is_healthy());
    cvlr_assert!(false);
}

#[rule]
pub fn record_borrow_asset_yield_distribution_integrity_1() {
    let amount = BorrowAssetAmount::nondet();
    let mut market = Market::nondet();

    let account_id = AccountId::nondet();

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
    }
    else {
        let _ = snapshot_yield_distribution_before.join(amount);
        cvlr_assert!(
            snapshot_yield_distribution_after == snapshot_yield_distribution_before);
    }
}


// #[near(serializers=[])]
// pub struct MyData {
//     x: U64,
//     y: U128
// }

// extern "C" {
//     pub fn deserialize_MyData(p: *const u8, size: usize) -> MyData;
//     pub fn serialize_MyData(p: &MyData, size: *mut usize, p: *mut [u8]);
// }

// impl borsh::BorshDeserialize for MyData {
//     fn deserialize_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
//         let mut v = Vec::new();
//         let bytes = reader.read_to_end(&mut v)?;
//         let pv: &[u8] = &v;
//         unsafe {
//             Ok(deserialize_MyData(pv.as_ptr(), bytes))
//         }
//     }
// }

// impl borsh::BorshSerialize for MyData {
//     fn serialize<W: std::io::Write>(&self, writer: &mut W) -> std::io::Result<()> {
//         serialize_MyData
//         Ok(())
//     }
// }

// #[rule]
// pub fn storage(k: u64, l: LookupMap<u64, MyData, Identity>) {
//     let (_key, _elt) = LookupMap::<u64, MyData, Identity>::load_element(&l.prefix, &k);
//     cvlr_assert!(false);
// }

// #[rule]
// pub fn lookup1(k: u64, l: LookupMap<u64, u64>) {
//     let _foo = l.get(&k);
//     cvlr_assert!(false);
// }

// #[rule]
// pub fn lookup2(k: AccountId, l: LookupMap<AccountId, u64>) {
//     let _foo = l.get(&k);
//     cvlr_assert!(false);
// }