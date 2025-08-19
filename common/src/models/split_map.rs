use std::{borrow::Borrow, cell::RefCell, marker::PhantomData};
use near_sdk::near;
use crate::models::templar_nondet::*;

pub trait ApplyRule {
    type I;
    type J;
    fn apply_rule(&mut self, i: Self::I, j: Self::J);
}

impl<K, V, T> ApplyRule for SplitMap<K, V, T> {
    type I = K;
    type J = Option<V>;

    fn apply_rule(&mut self, i: Self::I, v: Self::J) {
        self.the_x = i;
        self.the_v = v;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[near(serializers = [json, borsh])]
pub struct SplitMap<K, V, T> 
{
    pub(crate) the_x: K,
    pub(crate) the_v: Option<V>,
    pub(crate) bot: RefCell<V>,
    pub(crate) d: PhantomData<T> ,
}

pub struct SplitMapIterator<V> { p: PhantomData<V>}
impl <V:TemplarNondet> std::iter::Iterator for SplitMapIterator<V> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        if bool::nondet() { None } else { Some(V::nondet())}
    }
}
impl <K, V:TemplarNondet, T> std::iter::IntoIterator for &SplitMap<K, V, T> {
    type Item = V;

    type IntoIter = SplitMapIterator<V>;

    fn into_iter(self) -> Self::IntoIter {
        SplitMapIterator { p: PhantomData }
    }
}

impl <K, V: TemplarNondet, T> SplitMap<K, V, T> {
    pub fn iter(&self) -> SplitMapIterator<V> {
        self.into_iter()
    }
}

// impl <K, V: TemplarNondet, T> Iterator for &SplitMap<K, V, T> {
//     type Item = V;

//     fn next(&mut self) -> Option<Self::Item> {
//     }
// }

impl <K: Default, V: Default, T> Default for SplitMap<K, V, T> {
    fn default() -> Self {
        Self { the_x: Default::default(), the_v: None, bot: RefCell::new(Default::default()), d: PhantomData }
    }
}

impl <K: TemplarNondet, V: TemplarNondet, T> TemplarNondet for SplitMap<K, V, T> {
    fn nondet() -> Self {
        Self {
            the_x: TemplarNondet::nondet(),
            the_v: TemplarNondet::nondet(),
            bot: RefCell::new(TemplarNondet::nondet()),
            d: PhantomData,
            // 
        }
    }
}


impl <K, V, T> SplitMap<K, V, T>
where
    K: Eq + std::hash::Hash + TemplarNondet,
    V: TemplarNondet + Clone,
{
    pub fn split(&mut self, the_k: K) {
        self.the_x = the_k;
        self.the_v = if cvlr::nondet::<u8>() == 0 {
            None
        } else {
            Some(V::nondet())
        }
    }

    pub fn nondet_v(&self) -> Option<&V> {
        if cvlr::nondet::<u8>() == 0 {
            None
        } else {
            let r = self.bot.as_ptr();
            unsafe { 
                *r = V::nondet();
                let p: &V = &*r;
                p.nondet_option()
            }
        }
    }

    pub fn new(_d: T) -> Self {
        Self {
            d: PhantomData,
            the_x: K::nondet(),
            the_v: None,
            bot: RefCell::new(V::nondet())
        }
    }

    pub fn insert(&mut self, k: &K, v: &V) {
        if self.the_x.borrow() == k {
            self.the_v = Some(v.clone());
        } else {
            let _ = self.nondet_v();
        }
    }
}