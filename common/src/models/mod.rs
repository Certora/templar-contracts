#![cfg(feature = "certora_any")]
/// This module contains models of the data structures used by Templar. 
/// 
/// Taking a HashMap<K,V> as an example, we essentially 
/// model such a map as a struct that tracks a single, _distinguished_ key.
/// 
/// `pub struct HashMapModel<K,V> { the_key: V, the_value: Option<V>}`
/// 
/// (note that, in the interest of legibility, 
/// the pseudo-code below is incorrect with respect to borrowing and ownership rules)
/// 
/// ```
/// impl HashMapModel<K,V> {
///   // If this is the distinguished key, just return the value we are tracking.
///   // otherwise, return a random value
///   pub fn get(&self, k: K) -> Option<V> {
///     if self.the_key == k { 
///       self.the_value
///     } else {
///       nondeterministic_value()
///     }
///   }
/// 
///   pub fn set(&mut self, k: K, v: V)  {
///     if self.the_key == k { 
///       self.the_value = Some(v);
///     }
///   }
/// }
/// ```
/// 
/// The models all make use of `split_map`, which implements the structure described above
pub mod templar_nondet;

pub mod split_map;
pub mod hash_map;
pub mod vec;
pub mod vector;
pub mod unordered_map;
pub mod lookup_map;