use std::{cell::RefCell, marker::PhantomData};

use borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::{near, IntoStorageKey};
use cvlr::{cvlr_assert};

use crate::models::{split_map::SplitMap, templar_nondet::*};

#[near(serializers = [json, borsh])]
pub struct Vector<V: BorshDeserialize + BorshSerialize>(SplitMap<usize, V, near_sdk::store::Vector<V>>);
pub struct VectorIter<'a, V: 'a> { p: PhantomData<&'a V> }
pub struct IntoIter<V> { p: PhantomData<V> }

impl<V: TemplarNondet + BorshDeserialize + BorshSerialize> Default for Vector<V> {
    fn default() -> Self {
        Self (SplitMap{ the_x: Default::default(), the_v: None, bot: RefCell::new(V::nondet()), d: PhantomData })
    }
}

impl<V: Clone + BorshDeserialize + BorshSerialize> Clone for Vector<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl <V> Vector<V>
where
    V: TemplarNondet + Clone + BorshDeserialize + BorshSerialize
{
    pub fn new<S: IntoStorageKey>(_v: S) -> Self {
        Vector(SplitMap::new())
    }

    pub fn iter(&self) -> VectorIter<'_, V> {
        VectorIter { p: PhantomData }
    }

    pub fn get_mut(&mut self, index: u32) -> Option<&mut V> {
        if self.0.the_x == index as usize {
            self.0.the_v.as_mut()
        } else {
            self.0.nondet_v();
            let p = self.0.bot.get_mut();
            p.nondet_option_mut()
        }
    }

    pub fn flush(&mut self) {
        cvlr_assert!(false);
    }

    pub fn get(&self, i: u32) -> Option<&V>
    {
            if self.0.the_x == i as usize {
                self.0.the_v.as_ref()
            } else {
                self.0.nondet_v();
                unsafe {
                    let p: &V = &*self.0.bot.as_ptr();
                    p.nondet_option()
                }
            }
    }

    pub fn last_mut(&mut self) -> Option<&mut V> {
        nondet_choice!(
           self.0.the_v.as_mut(),
           self.0.bot.get_mut().nondet_option_mut(),
           None
        )
    }

    pub fn len(&self) -> u32 {
        u32::nondet()
    }

    pub fn is_empty(&self) -> bool {
        if self.0.the_v.is_some() { false } else { bool::nondet() }
    }

    pub fn push(&mut self, v: V) {
        nondet_choice!(
            self.0.the_v = Some(v),
            { self.0.bot.replace(TemplarNondet::nondet()); },
            {}
        );
    }
    pub fn pop(&mut self) -> Option<V> {
        nondet_choice!(
            { 
                let v = self.0.the_v.clone();
                self.0.the_v = None;
                v
            },
            { 
                TemplarNondet::nondet()
            }
        )
    }
}

impl <V: TemplarNondet + BorshDeserialize + BorshSerialize> TemplarNondet for Vector<V> {
    fn nondet() -> Self {
        Vector(TemplarNondet::nondet())
    }
}

impl <'a, V: TemplarNondet> Iterator for VectorIter<'a, V> {
    type Item = &'a V;

    fn next(&mut self) -> Option<Self::Item> {
        if bool::nondet() { None } else { cvlr_assert!(false) /* ABAKST: TODO */; None }
    }
}

impl <V: TemplarNondet> Iterator for IntoIter<V> {
    type Item = V;
    
    fn next(&mut self) -> Option<Self::Item> {
        TemplarNondet::nondet()
    }

}

impl<V: TemplarNondet + BorshDeserialize + BorshSerialize> FromIterator<V> for Vector<V> {
    fn from_iter<T: IntoIterator<Item = V>>(_iter: T) -> Self {
        TemplarNondet::nondet()
    }
}

impl<'a, T: TemplarNondet + BorshDeserialize + BorshSerialize> IntoIterator for &'a Vector<T> {
    type Item = &'a T;

    type IntoIter = VectorIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VectorIter { p: PhantomData }
    }
}

impl<T: TemplarNondet + BorshDeserialize + BorshSerialize> IntoIterator for Vector<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { p: PhantomData }
    }
}

impl <V: PartialEq + BorshDeserialize + BorshSerialize + PartialEq> PartialEq for Vector<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}


