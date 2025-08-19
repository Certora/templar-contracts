use std::borrow::Borrow;
use std::cell::UnsafeCell;
use std::collections::HashMap;

use models::templar_nondet::*;
use near_sdk::json_types::{U64, U128};
use near_sdk::store::key::Identity;
use near_sdk::store::LookupMap;
use near_sdk::near;

use cvlr::{cvlr_satisfy, rule};
use cvlr::cvlr_assert;
use near_sdk::AccountId;
use templar_common::models;
use templar_common::models::split_map::ApplyRule;

#[rule]
pub fn split_ok_1() {
    let i: u32 = TemplarNondet::nondet();
    let j: u32 = TemplarNondet::nondet();
    let v: u32 = TemplarNondet::nondet();

    let mut m: models::hash_map::HashMap<u32, u32> = models::hash_map::HashMap::nondet();
    m.apply_rule(i, None);
    let _ = m
        .entry(i)
        .or_insert(v);
    let v2 = m.get(&j);
    v2.map(|the_val| cvlr_satisfy!(*the_val == v));
    v2.map(|the_val| cvlr_satisfy!(*the_val != v));
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