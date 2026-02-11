
use std::ops::{Deref, DerefMut};

use templar_common::asset::{BorrowAsset, BorrowAssetAmount};
use templar_common::borrow::error::InitialBorrowError;
use templar_common::borrow::{BorrowPosition, InitialBorrow};
use templar_common::fee::Fee;
use templar_common::market::ValidAmountRange;
use templar_common::number::Decimal;

pub(crate) struct SimpleMarketConfiguration {
    pub borrow_range: ValidAmountRange<BorrowAsset>,
    pub borrow_asset_maximum_usage_ratio: Decimal,
    pub borrow_origination_fee: Fee<BorrowAsset>,
}

pub(crate) struct SimpleMarket {
    pub configuration: SimpleMarketConfiguration,
    pub borrow_asset_balance: BorrowAssetAmount,
    pub borrow_asset_deposited_active: BorrowAssetAmount,
    pub borrow_asset_deposited_incoming_total: BorrowAssetAmount,
    pub borrow_asset_borrowed_in_flight: BorrowAssetAmount,
    pub current_yield_distribution: BorrowAssetAmount,
    pub single_snapshot_maximum_interest_precomputed: Decimal,
}

impl SimpleMarket {
    fn total_incoming(&self) -> BorrowAssetAmount {
        self.borrow_asset_deposited_incoming_total
    }

    fn get_borrow_asset_available_to_borrow(&self) -> BorrowAssetAmount {
        let must_retain: BorrowAssetAmount = ((1u32
            - self.configuration.borrow_asset_maximum_usage_ratio)
            * Decimal::from(self.borrow_asset_deposited_active))
        .to_u128_ceil()
        .unwrap_or(0)
        .into();

        self.borrow_asset_balance
            .saturating_sub(self.total_incoming())
            .saturating_sub(must_retain)
    }

    fn single_snapshot_fee(&self, amount: BorrowAssetAmount) -> Option<BorrowAssetAmount> {
        (u128::from(amount) * self.single_snapshot_maximum_interest_precomputed)
            .to_u128_ceil()
            .map(Into::into)
    }

    fn record_borrow_asset_yield_distribution(&mut self, amount: BorrowAssetAmount) {
        if amount.is_zero() {
            return;
        }
        self.current_yield_distribution += amount;
    }
}

pub(crate) struct SimpleBorrowPositionRef<M> {
    market: M,
    position: BorrowPosition,
}

pub(crate) struct SimpleBorrowPositionGuard<'a>(SimpleBorrowPositionRef<&'a mut SimpleMarket>);

impl<M> SimpleBorrowPositionRef<M> {
    pub fn new(market: M, position: BorrowPosition) -> Self {
        Self { market, position }
    }
}

impl<'a> Deref for SimpleBorrowPositionGuard<'a> {
    type Target = SimpleBorrowPositionRef<&'a mut SimpleMarket>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for SimpleBorrowPositionGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> SimpleBorrowPositionGuard<'a> {
    pub(crate) fn new(market: &'a mut SimpleMarket, position: BorrowPosition) -> Self {
        Self(SimpleBorrowPositionRef::new(market, position))
    }

    pub(crate) fn record_borrow_initial(
        &mut self,
        amount: BorrowAssetAmount,
    ) -> Result<InitialBorrow, InitialBorrowError> {
        let available_to_borrow = self.market.get_borrow_asset_available_to_borrow();
        if amount > available_to_borrow {
            return Err(InitialBorrowError::InsufficientBorrowAssetAvailable);
        }

        let origination_fee = self
            .market
            .configuration
            .borrow_origination_fee
            .of(amount)
            .ok_or(InitialBorrowError::FeeCalculationFailure)?;

        let single_snapshot_fee = self
            .market
            .single_snapshot_fee(amount)
            .ok_or(InitialBorrowError::FeeCalculationFailure)?;

        let origination_u = u128::from(origination_fee);
        let snapshot_u = u128::from(single_snapshot_fee);
        if origination_u > u128::MAX - snapshot_u {
            return Err(InitialBorrowError::FeeCalculationFailure);
        }
        let fees = BorrowAssetAmount::from(origination_u + snapshot_u);

        self.market.borrow_asset_borrowed_in_flight += amount;
        self.position.borrow_asset_in_flight += amount;
        self.position.fees += fees;

        if !self.within_allowable_borrow_range() {
            self.market.borrow_asset_borrowed_in_flight -= amount;
            self.position.borrow_asset_in_flight -= amount;
            self.position.fees -= fees;
            return Err(InitialBorrowError::OutsideAllowableRange);
        }

        self.market.record_borrow_asset_yield_distribution(fees);
        self.market.borrow_asset_balance -= amount;

        Ok(InitialBorrow { amount, fees })
    }

    pub(crate) fn borrow_asset_principal(&self) -> BorrowAssetAmount {
        self.position.get_borrow_asset_principal()
    }
}

impl<M: Deref<Target = SimpleMarket>> SimpleBorrowPositionRef<M> {
    pub fn within_allowable_borrow_range(&self) -> bool {
        self.market
            .configuration
            .borrow_range
            .contains(self.position.get_borrow_asset_principal())
    }

}
