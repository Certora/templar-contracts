use std::str::FromStr;

use near_sdk::{json_types::U128, AccountId};

use crate::{accumulator::Accumulator, asset::{BorrowAsset, FungibleAssetAmount}, supply::{IncomingDeposit, SupplyPosition}};

// Abakst: move these to CVLR?
macro_rules! make_project_nondet_ {
    [$name:tt, $dollar:tt] => {
            pub trait $name { fn nondet() -> Self; }
            macro_rules! declare_nondet {
                (from_nondet, $t:ty) => {
                    impl $name for $t { 
                        fn nondet() -> Self { cvlr::nondet::nondet() }
                    }
                };
                (option) => {
                    impl <T:$name> $name for Option<T> {
                        fn nondet() -> Self {
                            if bool::nondet() { None } else { Some(T::nondet()) }
                        }
                    }
                };
                ($t:ty, $e:expr) => {
                    impl $name for $t {
                        fn nondet() -> Self {
                            $e
                        }
                    }
                };
                ($t:ty, $dollar ( $x : tt ),*  => $e:expr) => {
                    impl $name for $t {
                        fn nondet() -> Self {
                            $dollar( let $x = $name::nondet(); )*
                            $e
                        }
                    }
                };
            }
    };
}

macro_rules! make_project_nondet {
    ($name:tt) => { make_project_nondet_![$name, $]; };
}

make_project_nondet!(TemplarNondet);

declare_nondet!(from_nondet, u8);
declare_nondet!(from_nondet, u16);
declare_nondet!(from_nondet, u32);
declare_nondet!(from_nondet, u64);
declare_nondet!(from_nondet, u128);
declare_nondet!(from_nondet, usize);
declare_nondet!(from_nondet, i8);
declare_nondet!(from_nondet, i16);
declare_nondet!(from_nondet, i32);
declare_nondet!(from_nondet, i128);
declare_nondet!(from_nondet, bool);
declare_nondet!(from_nondet, FungibleAssetAmount<BorrowAsset>);
declare_nondet!(
    Accumulator<BorrowAsset>,
    total,
    fraction_as_u128_dividend,
    next_snapshot_index,
    pending_estimate,
    amortized =>
    Accumulator::new_raw(total, fraction_as_u128_dividend, next_snapshot_index, pending_estimate, amortized)
);
declare_nondet!(IncomingDeposit, amount, activate_at_snapshot_index => IncomingDeposit { amount, activate_at_snapshot_index });
declare_nondet!(option);
declare_nondet!(AccountId, FromStr::from_str("nondet").unwrap());
declare_nondet!(near_sdk::json_types::U64, x => near_sdk::json_types::U64(x));

declare_nondet!(U128, y => U128(y));

declare_nondet!(
    crate::supply::Deposit,
    active, incoming, outgoing => 
    crate::supply::Deposit { active, incoming, outgoing }
);

declare_nondet!(
    SupplyPosition, 
    started_at_block_timestamp_ms, borrow_asset_deposit, borrow_asset_yield => 
    SupplyPosition::new_raw(started_at_block_timestamp_ms, borrow_asset_yield, borrow_asset_deposit)
);

pub trait LiftOption {
    fn nondet_option(&self) -> Option<&Self> {
        if bool::nondet() { None } else { Some(self) }
    }
    fn nondet_option_mut(&mut self) -> Option<&mut Self> {
        if bool::nondet() { None } else { Some(self) }
    }
}

impl <T> LiftOption for T {}