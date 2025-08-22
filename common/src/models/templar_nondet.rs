use std::{alloc::GlobalAlloc, io::Read, num::{NonZeroU16, NonZeroU32}, str::{from_utf8_unchecked, FromStr}};

use cvlr::cvlr_assume;
use near_sdk::{json_types::U128, AccountId};

use crate::{accumulator::Accumulator, asset::{BorrowAsset, CollateralAsset, FungibleAssetAmount}, borrow::BorrowPosition, static_yield::StaticYieldRecord, supply::{IncomingDeposit, SupplyPosition}};

#[inline(never)]
pub fn certora_ite<E>(b: bool, tt: E, ff: E) -> E {
    if b { tt } else { ff }
}

pub fn certora_choose<E>(tt: E, ff: E) -> E {
    if bool::nondet() { tt } else { ff }
}

macro_rules! nondet_choice {
    ($e1:expr) => { $e1 };
    ($e1:expr, $( $e:expr ),*) => { crate::models::templar_nondet::certora_choose($e1, nondet_choice!($( $e ),*)) };
}
pub(crate) use nondet_choice;

// Abakst: move these to CVLR?
macro_rules! make_project_nondet_ {
    [$name:tt, $dollar:tt] => {
            pub trait $name { fn nondet() -> Self; }
            impl <A: $name, B: $name> $name for (A, B) {
                fn nondet() -> Self {
                    (A::nondet(), B::nondet())
                }
            }
            macro_rules! declare_nondet {
                (from_nondet, $t:ty) => {
                    impl $name for $t { 
                        fn nondet() -> Self { cvlr::nondet::nondet() }
                    }
                };
                (option) => {
                    impl <T:$name> $name for Option<T> {
                        fn nondet() -> Self {
                            certora_choose(None, Some(T::nondet()))
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
        pub(crate) use declare_nondet;
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
declare_nondet!(
    [u8; 32],
    {
        let mut whole = [0u8; 32];
        let (first, second) = whole.split_at_mut(16);
        first.copy_from_slice(&u128::nondet().to_le_bytes());
        second.copy_from_slice(&u128::nondet().to_le_bytes());
        whole
    }
);
declare_nondet!(
    [u64; 8],
    {
        let mut whole = [0u64; 8];
        unsafe {
            let pwhole: *mut u8 = whole.as_mut_ptr().cast();
            let bytes = CERTORA_nondet_bytes(64);
            std::ptr::copy_nonoverlapping(
                bytes, 
                pwhole, 
                64,
            ); 
        }
        whole
    }
);
declare_nondet!(primitive_types::U512, primitive_types::U512(TemplarNondet::nondet()));
// declare_nondet!(from_nondet, FungibleAssetAmount<BorrowAsset>);
// declare_nondet!(from_nondet, FungibleAssetAmount<CollateralAsset>);
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
// declare_nondet!(AccountId, 
//     {
//         let foo = Box<String::from("asdf").into_boxed_str()>;

//     }
// //    FromStr::from_str("nondet").unwrap()
// );
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

declare_nondet!(
    BorrowPosition,
            started_at_block_timestamp_ms,
            collateral_asset_deposit,
            borrow_asset_principal,
            borrow_asset_fees,
            temporary_lock,
            is_liquidation_locked =>
            BorrowPosition::new_raw(
            started_at_block_timestamp_ms,
            collateral_asset_deposit,
            borrow_asset_principal,
            borrow_asset_fees,
            temporary_lock,
            is_liquidation_locked
            )
        );

declare_nondet!(
    StaticYieldRecord,
    collateral_asset, borrow_asset =>
    StaticYieldRecord { collateral_asset, borrow_asset }
);


declare_nondet!(
    NonZeroU16,
    {
        let x = u16::nondet();
        cvlr::cvlr_assume!(x != 0);
        unsafe { NonZeroU16::new_unchecked(x) }
    }
);

declare_nondet!(
    NonZeroU32,
    {
        let x = u32::nondet();
        cvlr::cvlr_assume!(x != 0);
        unsafe { NonZeroU32::new_unchecked(x) }
    }
);

pub fn nondet_bytes_sz(sz: usize) -> String {
    unsafe {
        let bytes = CERTORA_nondet_bytes(sz as u32);
        String::from_raw_parts(bytes, sz, sz)
    }
}


impl TemplarNondet for Box<str> {
    #[inline(never)]
    fn nondet() -> Self {
        // let sz = usize::nondet();
        // cvlr_assume!(AccountId::MIN_LEN <= sz && sz <= AccountId::MAX_LEN);
        let sz = 2;
        let s = nondet_bytes_sz(sz);
        id(s.into_boxed_str())
    }
}

#[inline(never)]
fn id<A>(a: A) -> A { a }

impl TemplarNondet for AccountId {
    #[inline(never)]
    fn nondet() -> Self {
        unsafe {
            let bstr = Box::<str>::nondet();
            std::mem::transmute(bstr)
        }
    }
}

unsafe extern "C" {
    pub unsafe fn CERTORA_nondet_bytes(n: u32) -> *mut u8;
}

pub trait LiftOption {
    fn nondet_option(&self) -> Option<&Self> {
        certora_choose(None, Some(self) )
    }
    fn nondet_option_mut(&mut self) -> Option<&mut Self> {
        certora_choose(None, Some(self) )
    }
}

impl <T> LiftOption for T {}