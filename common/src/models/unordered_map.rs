use std::marker::PhantomData;

use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::near;

use crate::models::{split_map::SplitMap, templar_nondet::*};

#[near(serializers = [json, borsh])]
pub struct UnorderedMap<K: BorshSerialize + BorshDeserialize, V: BorshSerialize + BorshDeserialize>(
    SplitMap<K,V,near_sdk::collections::UnorderedMap<K,V>>
);

impl<K: BorshSerialize + BorshDeserialize + TemplarNondet, V: BorshSerialize + BorshDeserialize + TemplarNondet> TemplarNondet for UnorderedMap<K, V> {
    fn nondet() -> Self {
        UnorderedMap(TemplarNondet::nondet())
    }
}

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

impl <K, V> UnorderedMap<K,V> 
where
    K: std::hash::Hash + Eq + TemplarNondet + BorshSerialize + BorshDeserialize,
    V: Clone + TemplarNondet + BorshSerialize + BorshDeserialize,
{
    pub fn new(prefix: std::vec::Vec<u8>) -> Self {
        UnorderedMap(SplitMap::new(near_sdk::collections::UnorderedMap::new(prefix)))
    }

    pub fn focus(&mut self, k: K) {
        self.0.split(k);
    }


    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter::default()
    }

    #[inline(never)]
    pub fn get(&self, k: &K) -> Option<V> {
        let have = self.0.the_v.clone();
        let have_not = TemplarNondet::nondet();
        if self.0.the_x.eq(k) { 
            have
        } else { 
            have_not
        }
    }

    pub fn remove(&mut self, _k: &K) -> Option<V> {
        TemplarNondet::nondet()
    }

    pub fn insert(&mut self, k: &K, v: &V) {
        if self.0.the_x == *k {
            self.0.the_v.replace(v.clone());
        }
    }
}