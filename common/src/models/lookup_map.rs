use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::near;

use crate::models::{split_map::SplitMap, templar_nondet::*};

#[near(serializers = [json, borsh])]
pub struct LookupMap<K: BorshSerialize + BorshDeserialize, V: BorshSerialize + BorshDeserialize>(
    SplitMap<K,V,near_sdk::collections::LookupMap<K,V>>
);

pub struct Iter<'a, K: 'a, V: 'a>(PhantomData<&'a (K, V)>);

impl<K, V> Default for Iter<'_, K, V> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl <'a, K, V> Iterator for Iter<'a, K, V> 
where 
    K: 'a + TemplarNondet,
    V: 'a + TemplarNondet,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        TemplarNondet::nondet()
    }
}

impl <K, V> LookupMap<K,V> 
where
    K: std::hash::Hash + Eq + TemplarNondet + BorshSerialize + BorshDeserialize,
    V: Clone + TemplarNondet + BorshSerialize + BorshDeserialize,
{
    pub fn new(prefix: std::vec::Vec<u8>) -> Self {
        LookupMap(SplitMap::new(near_sdk::collections::LookupMap::new(prefix)))
    }

    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::default()
    }

    pub fn get(&self, _k: &K) -> Option<V> {
        TemplarNondet::nondet()
    }

    pub fn remove(&mut self, _k: &K) -> Option<V> {
        TemplarNondet::nondet()
    }

    pub fn insert(&mut self, _k: &K, _v: &V) {

    }
}