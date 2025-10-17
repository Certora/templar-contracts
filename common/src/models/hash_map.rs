use std::{borrow::Borrow, cell::RefCell, marker::PhantomData};
use near_sdk::near;
use crate::models::{split_map::{ApplyRule, SplitMap}, templar_nondet::TemplarNondet};


#[derive(Eq, PartialEq, Clone, Debug)]
#[near(serializers = [json, borsh])]
pub struct HashMap<K: Eq + std::hash::Hash, V: Eq>(SplitMap<K, V, std::collections::HashMap<K,V>>);
pub struct Values<'a, K: 'a, V: 'a> { p: PhantomData<(&'a K, V)> }

impl<'a, K: 'a + TemplarNondet, V: 'a + TemplarNondet> Values<'a, K, V> {
    pub fn try_fold<F, B: TemplarNondet>(&mut self, _init: B, _f: F) -> Option<B>
    where
        Self: Sized,
        F: FnMut(B, &'a V) -> Option<B>,
    {
        TemplarNondet::nondet()
    }
}

impl <K: Eq + std::hash::Hash + TemplarNondet, V: Eq + TemplarNondet> TemplarNondet for HashMap<K, V> {
    fn nondet() -> Self {
        HashMap(TemplarNondet::nondet())
    }
}

impl<K: Eq + std::hash::Hash, V: Eq> ApplyRule for HashMap<K, V> {
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
pub struct Entry<'a, K:'a + Eq + std::hash::Hash, V: 'a + Eq>(&'a mut HashMap<K, V>, K);

impl <'a, K, V> Entry<'a, K, V> 
where
    K: 'a + Eq + std::hash::Hash + TemplarNondet,
    V: 'a + TemplarNondet + Clone + Eq,
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
        }
        self
    }

}

pub struct IntoIter<'a, K: 'a + TemplarNondet, V: 'a + TemplarNondet>(RefCell<K>, RefCell<V>, PhantomData<&'a K>);

impl<'a, K: 'a + TemplarNondet, V: 'a + TemplarNondet> Iterator for IntoIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        if bool::nondet() {
            None
        } else {
            self.0.replace(K::nondet());
            self.1.replace(V::nondet());
            unsafe {
                Some((&*self.0.as_ptr(), &*self.1.as_ptr()))
            }
        }
    }
}


impl<'a, K: Eq + std::hash::Hash + TemplarNondet, V: TemplarNondet + Eq> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);

    type IntoIter = IntoIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(RefCell::new(K::nondet()), RefCell::new(V::nondet()), PhantomData)
    }
}

impl <K,V> HashMap<K,V>
where
    K: Eq + std::hash::Hash + TemplarNondet,
    V: Eq + TemplarNondet + Clone,
{
    pub fn new() -> Self {
        Self(
            SplitMap::new()
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

    pub fn insert(&mut self, k: K, v: V) {
        if self.0.the_x == k {
            self.0.the_v = Some(v);
        }
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
            TemplarNondet::nondet()
        }
    }

}
