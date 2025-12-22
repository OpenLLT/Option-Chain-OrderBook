//! Strike order book module.
//!
//! This module provides the [`StrikeOrderBook`] and [`StrikeOrderBookManager`]
//! for managing call/put pairs at a specific strike price.

use super::book::OptionOrderBook;
use super::quote::Quote;
use crate::error::{Error, Result};
use crate::utils::format_expiration_yyyymmdd;
use dashmap::DashMap;
use optionstratlib::greeks::Greek;
use optionstratlib::{ExpirationDate, OptionStyle};

/// Order book for a single strike price containing both call and put.
///
/// This struct manages the call/put pair at a specific strike price.
///
/// ## Architecture
///
/// ```text
/// StrikeOrderBook (per strike price)
///   ├── OptionOrderBook (call)
///   │     └── OrderBook<T> (from OrderBook-rs)
///   └── OptionOrderBook (put)
///         └── OrderBook<T> (from OrderBook-rs)
/// ```
pub struct StrikeOrderBook {
    /// The underlying asset symbol (e.g., "BTC").
    underlying: String,
    /// The expiration date.
    expiration: ExpirationDate,
    /// The strike price.
    strike: u64,
    /// Call option order book.
    call: OptionOrderBook,
    /// Put option order book.
    put: OptionOrderBook,
    /// Greeks for the call option.
    call_greeks: Option<Greek>,
    /// Greeks for the put option.
    put_greeks: Option<Greek>,
}

impl StrikeOrderBook {
    /// Creates a new strike order book.
    ///
    /// # Arguments
    ///
    /// * `underlying` - The underlying asset symbol (e.g., "BTC")
    /// * `expiration` - The expiration date
    /// * `strike` - The strike price
    #[must_use]
    pub fn new(underlying: impl Into<String>, expiration: ExpirationDate, strike: u64) -> Self {
        let underlying = underlying.into();

        // Format expiration as YYYYMMDD, fallback to Display if formatting fails
        let exp_str =
            format_expiration_yyyymmdd(&expiration).unwrap_or_else(|_| expiration.to_string());

        let call_symbol = format!("{}-{}-{}-C", underlying, exp_str, strike);
        let put_symbol = format!("{}-{}-{}-P", underlying, exp_str, strike);

        Self {
            underlying,
            expiration,
            strike,
            call: OptionOrderBook::new(call_symbol, OptionStyle::Call),
            put: OptionOrderBook::new(put_symbol, OptionStyle::Put),
            call_greeks: None,
            put_greeks: None,
        }
    }

    /// Returns the underlying asset symbol.
    #[must_use]
    pub fn underlying(&self) -> &str {
        &self.underlying
    }

    /// Returns the expiration date.
    #[must_use]
    pub const fn expiration(&self) -> &ExpirationDate {
        &self.expiration
    }

    /// Returns the strike price.
    #[must_use]
    pub const fn strike(&self) -> u64 {
        self.strike
    }

    /// Returns a reference to the call order book.
    #[must_use]
    pub const fn call(&self) -> &OptionOrderBook {
        &self.call
    }

    /// Returns a mutable reference to the call order book.
    pub fn call_mut(&mut self) -> &mut OptionOrderBook {
        &mut self.call
    }

    /// Returns a reference to the put order book.
    #[must_use]
    pub const fn put(&self) -> &OptionOrderBook {
        &self.put
    }

    /// Returns a mutable reference to the put order book.
    pub fn put_mut(&mut self) -> &mut OptionOrderBook {
        &mut self.put
    }

    /// Returns the order book for the specified option style.
    #[must_use]
    pub const fn get(&self, option_style: OptionStyle) -> &OptionOrderBook {
        match option_style {
            OptionStyle::Call => &self.call,
            OptionStyle::Put => &self.put,
        }
    }

    /// Returns a mutable reference to the order book for the specified option style.
    pub fn get_mut(&mut self, option_style: OptionStyle) -> &mut OptionOrderBook {
        match option_style {
            OptionStyle::Call => &mut self.call,
            OptionStyle::Put => &mut self.put,
        }
    }

    /// Returns the best quote for the call option.
    #[must_use]
    pub fn call_quote(&self) -> Quote {
        self.call.best_quote()
    }

    /// Returns the best quote for the put option.
    #[must_use]
    pub fn put_quote(&self) -> Quote {
        self.put.best_quote()
    }

    /// Returns true if both call and put have two-sided quotes.
    #[must_use]
    pub fn is_fully_quoted(&self) -> bool {
        self.call.best_quote().is_two_sided() && self.put.best_quote().is_two_sided()
    }

    /// Returns the total order count across call and put.
    #[must_use]
    pub fn order_count(&self) -> usize {
        self.call.order_count() + self.put.order_count()
    }

    /// Returns true if both call and put are empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.call.is_empty() && self.put.is_empty()
    }

    /// Clears all orders from both call and put books.
    pub fn clear(&self) {
        self.call.clear();
        self.put.clear();
    }

    /// Updates the Greeks for the call option.
    pub fn update_call_greeks(&mut self, greeks: Greek) {
        self.call_greeks = Some(greeks);
    }

    /// Updates the Greeks for the put option.
    pub fn update_put_greeks(&mut self, greeks: Greek) {
        self.put_greeks = Some(greeks);
    }

    /// Returns the Greeks for the call option.
    #[must_use]
    pub const fn call_greeks(&self) -> Option<&Greek> {
        self.call_greeks.as_ref()
    }

    /// Returns the Greeks for the put option.
    #[must_use]
    pub const fn put_greeks(&self) -> Option<&Greek> {
        self.put_greeks.as_ref()
    }
}

/// Manages strike order books for a single expiration.
///
/// Provides centralized access to all strikes within an expiration.
/// Uses `DashMap` for thread-safe concurrent access.
pub struct StrikeOrderBookManager {
    /// Strike order books indexed by strike price.
    strikes: DashMap<u64, StrikeOrderBook>,
    /// The underlying asset symbol.
    underlying: String,
    /// The expiration date.
    expiration: ExpirationDate,
}

impl StrikeOrderBookManager {
    /// Creates a new strike order book manager.
    ///
    /// # Arguments
    ///
    /// * `underlying` - The underlying asset symbol
    /// * `expiration` - The expiration date
    #[must_use]
    pub fn new(underlying: impl Into<String>, expiration: ExpirationDate) -> Self {
        Self {
            strikes: DashMap::new(),
            underlying: underlying.into(),
            expiration,
        }
    }

    /// Returns the underlying asset symbol.
    #[must_use]
    pub fn underlying(&self) -> &str {
        &self.underlying
    }

    /// Returns the expiration date.
    #[must_use]
    pub const fn expiration(&self) -> &ExpirationDate {
        &self.expiration
    }

    /// Returns the number of strikes.
    #[must_use]
    pub fn len(&self) -> usize {
        self.strikes.len()
    }

    /// Returns true if there are no strikes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.strikes.is_empty()
    }

    /// Gets or creates a strike order book, returning a guard for access.
    pub fn get_or_create(
        &self,
        strike: u64,
    ) -> dashmap::mapref::one::Ref<'_, u64, StrikeOrderBook> {
        self.strikes
            .entry(strike)
            .or_insert_with(|| StrikeOrderBook::new(&self.underlying, self.expiration, strike))
            .downgrade()
    }

    /// Gets a strike order book by strike price.
    ///
    /// # Errors
    ///
    /// Returns `Error::StrikeNotFound` if the strike does not exist.
    pub fn get(&self, strike: u64) -> Result<dashmap::mapref::one::Ref<'_, u64, StrikeOrderBook>> {
        self.strikes
            .get(&strike)
            .ok_or_else(|| Error::strike_not_found(strike))
    }

    /// Returns true if a strike exists.
    #[must_use]
    pub fn contains(&self, strike: u64) -> bool {
        self.strikes.contains_key(&strike)
    }

    /// Removes a strike order book.
    ///
    /// Note: Returns true if the strike was removed, false if it didn't exist.
    pub fn remove(&self, strike: u64) -> bool {
        self.strikes.remove(&strike).is_some()
    }

    /// Returns all strike prices (sorted).
    pub fn strike_prices(&self) -> Vec<u64> {
        let mut prices: Vec<u64> = self.strikes.iter().map(|e| *e.key()).collect();
        prices.sort_unstable();
        prices
    }

    /// Returns the total order count across all strikes.
    #[must_use]
    pub fn total_order_count(&self) -> usize {
        self.strikes.iter().map(|e| e.value().order_count()).sum()
    }

    /// Returns the ATM (at-the-money) strike closest to the given spot price.
    ///
    /// # Errors
    ///
    /// Returns `Error::NoDataAvailable` if there are no strikes.
    pub fn atm_strike(&self, spot: u64) -> Result<u64> {
        self.strikes
            .iter()
            .map(|e| *e.key())
            .min_by_key(|&k| (k as i64 - spot as i64).unsigned_abs())
            .ok_or_else(|| Error::no_data("no strikes available"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optionstratlib::pos;
    use orderbook_rs::{OrderId, Side};

    fn test_expiration() -> ExpirationDate {
        ExpirationDate::Days(pos!(30.0))
    }

    #[test]
    fn test_strike_order_book_creation() {
        let strike = StrikeOrderBook::new("BTC", test_expiration(), 50000);

        assert_eq!(strike.underlying(), "BTC");
        assert_eq!(strike.strike(), 50000);
        assert!(strike.is_empty());
    }

    #[test]
    fn test_strike_order_book_orders() {
        let strike = StrikeOrderBook::new("BTC", test_expiration(), 50000);

        strike
            .call()
            .add_limit_order(OrderId::new(), Side::Buy, 100, 10)
            .unwrap();
        strike
            .put()
            .add_limit_order(OrderId::new(), Side::Sell, 50, 5)
            .unwrap();

        assert_eq!(strike.order_count(), 2);
        assert!(!strike.is_empty());
    }

    #[test]
    fn test_strike_manager_creation() {
        let manager = StrikeOrderBookManager::new("BTC", test_expiration());

        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert_eq!(manager.underlying(), "BTC");
    }

    #[test]
    fn test_strike_manager_get_or_create() {
        let manager = StrikeOrderBookManager::new("BTC", test_expiration());

        let strike = manager.get_or_create(50000);
        assert_eq!(strike.strike(), 50000);

        manager.get_or_create(55000);
        manager.get_or_create(45000);

        assert_eq!(manager.len(), 3);

        let strikes = manager.strike_prices();
        assert_eq!(strikes, vec![45000, 50000, 55000]);
    }

    #[test]
    fn test_strike_manager_atm() {
        let manager = StrikeOrderBookManager::new("BTC", test_expiration());

        manager.get_or_create(45000);
        manager.get_or_create(50000);
        manager.get_or_create(55000);

        assert_eq!(manager.atm_strike(48000).unwrap(), 50000);
        assert_eq!(manager.atm_strike(52000).unwrap(), 50000);
        assert_eq!(manager.atm_strike(53000).unwrap(), 55000);
    }

    #[test]
    fn test_strike_manager_atm_empty() {
        let manager = StrikeOrderBookManager::new("BTC", test_expiration());
        assert!(manager.atm_strike(50000).is_err());
    }
}
