use std::{borrow::Borrow, marker::PhantomData};
use near_sdk::near;
use crate::models::{split_map::{ApplyRule, SplitMap}, templar_nondet::TemplarNondet};


#[derive(Clone, Debug)]
#[near(serializers = [json, borsh])]
pub struct HashMap<K, V>(SplitMap<K, V, std::collections::HashMap<K,V>>);
pub struct Values<'a, K: 'a, V: 'a> { p: PhantomData<(&'a K, V)> }

impl <K: TemplarNondet, V: TemplarNondet> TemplarNondet for HashMap<K, V> {
    fn nondet() -> Self {
        HashMap(TemplarNondet::nondet())
    }
}

impl<K, V> ApplyRule for HashMap<K, V> {
    type I = K;
    type J = Option<V>;

    fn apply_rule(&mut self, i: Self::I, j: Self::J) {
        self.0.apply_rule(i, j);
    }
}

impl <'a, K, V> Values<'a, K, V> {
    pub fn fold<B: TemplarNondet, F>(self, _init: B, _f: F) -> B
    where
        F: FnMut(B, &'a V) -> B
    {
        B::nondet()
    }
}
pub struct Entry<'a, K:'a , V: 'a>(&'a mut HashMap<K, V>, K);

impl <'a, K, V> Entry<'a, K, V> 
where
    K: 'a + Eq + std::hash::Hash + TemplarNondet,
    V: 'a + TemplarNondet + Clone,
{
    pub fn or_insert(self, default: V) -> &'a mut V {
        if self.1 == self.0.0.the_x {
            self.0.0.the_v.get_or_insert(default)
        } else {
            self.0.0.nondet_v();
            self.0.0.bot.get_mut()
        }
    }

    pub fn and_modify<F>(self, f: F) -> Self 
    where
        F : FnOnce(&mut V)
    {
        if self.1 == self.0.0.the_x {
            if let Some(p) = self.0.0.the_v.as_mut() {
                f(p);
            }
        } else {
            self.0.0.nondet_v();
        }
        self
    }

}

impl <K,V> HashMap<K,V>
where
    K: Eq + std::hash::Hash + TemplarNondet,
    V: TemplarNondet + Clone,
{
    pub fn new(d: std::collections::HashMap<K,V>) -> Self {
        Self(
            SplitMap::new(d)
        )

    }

    pub fn values(&self) -> Values<'_, K, V> {
        Values { p:PhantomData }
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V> 
    where
        K: Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        if self.0.the_x.borrow() == k {
            self.0.the_v.as_ref()
        } else {
            self.0.nondet_v()
        }
    }

    pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
        Entry(self, key)
    }


    pub fn remove<Q: ?Sized>(&mut self, k: &Q) -> Option<V> 
    where
        K: Borrow<Q>,
        Q: std::hash::Hash + Eq,
    {
        if self.0.the_x.borrow() == k {
            let r = self.0.the_v.as_ref().cloned();
            self.0.the_v = None;
            r
        } else {
            self.0.nondet_v().cloned()
        }
    }

}
