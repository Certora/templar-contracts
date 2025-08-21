use std::{cell::RefCell, marker::PhantomData};

use near_sdk::near;
use cvlr::cvlr_assert;

use crate::models::{split_map::SplitMap, templar_nondet::*};

#[derive(Clone, Debug, Eq)]
#[near(serializers = [json, borsh])]
pub struct Vec<V>(SplitMap<usize, V, std::vec::Vec<V>>);
pub struct VecIter<'a, V: 'a> { p: PhantomData<&'a V> }
pub struct IntoIter<V> { p: PhantomData<V> }

impl<V: TemplarNondet> Default for Vec<V> {
    fn default() -> Self {
        Self (SplitMap{ the_x: Default::default(), the_v: None, bot: RefCell::new(V::nondet()), d: PhantomData })
    }
}

impl <V> Vec<V>
where
    V: TemplarNondet + Clone
{
    pub fn new(v: std::vec::Vec<V>) -> Self {
        Vec(SplitMap::new(v))
    }

    pub fn iter(&self) -> VecIter<'_, V> {
        VecIter { p: PhantomData }
    }

    pub fn get<I>(&self, index: I) -> Option<&V>
    where
        usize: TryFrom<I>
    {
        if let Ok(i) = usize::try_from(index) {
            if self.0.the_x == i {
                self.0.the_v.as_ref()
            } else {
             None // abakst fix
            }
        } else {
            None
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut V> {
        // ABAKST can these be ITEs?
        nondet_choice!(
            self.0.the_v.as_mut(),
            self.0.bot.get_mut().nondet_option_mut(),
            None
        )
    }

    pub fn len(&self) -> usize {
        usize::nondet()
    }

    pub fn push(&mut self, v: V) {
        nondet_choice!(
            self.0.the_v = Some(v),
            { self.0.bot.replace(TemplarNondet::nondet()); },
            {}
        )
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

impl <V: TemplarNondet> TemplarNondet for Vec<V> {
    fn nondet() -> Self {
        Vec(TemplarNondet::nondet())
    }
}

impl <'a, V: TemplarNondet> Iterator for VecIter<'a, V> {
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

impl<V: TemplarNondet> FromIterator<V> for Vec<V> {
    fn from_iter<T: IntoIterator<Item = V>>(_iter: T) -> Self {
        TemplarNondet::nondet()
    }
}

impl<'a, T: TemplarNondet> IntoIterator for &'a Vec<T> {
    type Item = &'a T;

    type IntoIter = VecIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        VecIter { p: PhantomData }
    }
}

impl<T: TemplarNondet> IntoIterator for Vec<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { p: PhantomData }
    }
}

impl <V: PartialEq> PartialEq for Vec<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}


