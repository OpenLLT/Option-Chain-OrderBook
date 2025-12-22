//! Expiration order book module.
//!
//! This module provides the [`ExpirationOrderBook`] and [`ExpirationOrderBookManager`]
//! for managing all expirations for a single underlying asset.

use super::chain::OptionChainOrderBook;
use super::strike::StrikeOrderBook;
use crate::error::{Error, Result};
use dashmap::DashMap;
use optionstratlib::ExpirationDate;

/// Order book for a single expiration date.
///
/// Contains the complete option chain for a specific expiration.
///
/// ## Architecture
///
/// ```text
/// ExpirationOrderBook (per expiry date)
///   └── OptionChainOrderBook
///         └── StrikeOrderBookManager
///               └── StrikeOrderBook (per strike)
/// ```
pub struct ExpirationOrderBook {
    /// The underlying asset symbol.
    underlying: String,
    /// The expiration date.
    expiration: ExpirationDate,
    /// The option chain for this expiration.
    chain: OptionChainOrderBook,
}

impl ExpirationOrderBook {
    /// Creates a new expiration order book.
    ///
    /// # Arguments
    ///
    /// * `underlying` - The underlying asset symbol (e.g., "BTC")
    /// * `expiration` - The expiration date
    #[must_use]
    pub fn new(underlying: impl Into<String>, expiration: ExpirationDate) -> Self {
        let underlying = underlying.into();

        Self {
            chain: OptionChainOrderBook::new(&underlying, expiration),
            underlying,
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

    /// Returns a reference to the option chain.
    #[must_use]
    pub const fn chain(&self) -> &OptionChainOrderBook {
        &self.chain
    }

    /// Gets or creates a strike order book, returning a guard for access.
    pub fn get_or_create_strike(
        &self,
        strike: u64,
    ) -> dashmap::mapref::one::Ref<'_, u64, StrikeOrderBook> {
        self.chain.get_or_create_strike(strike)
    }

    /// Gets a strike order book.
    ///
    /// # Errors
    ///
    /// Returns `Error::StrikeNotFound` if the strike does not exist.
    pub fn get_strike(
        &self,
        strike: u64,
    ) -> Result<dashmap::mapref::one::Ref<'_, u64, StrikeOrderBook>> {
        self.chain.get_strike(strike)
    }

    /// Returns the number of strikes.
    #[must_use]
    pub fn strike_count(&self) -> usize {
        self.chain.strike_count()
    }

    /// Returns true if there are no strikes.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }

    /// Returns all strike prices (sorted).
    pub fn strike_prices(&self) -> Vec<u64> {
        self.chain.strike_prices()
    }

    /// Returns the total order count.
    #[must_use]
    pub fn total_order_count(&self) -> usize {
        self.chain.total_order_count()
    }

    /// Returns the ATM strike closest to the given spot price.
    ///
    /// # Errors
    ///
    /// Returns `Error::NoDataAvailable` if there are no strikes.
    pub fn atm_strike(&self, spot: u64) -> Result<u64> {
        self.chain.atm_strike(spot)
    }
}

/// Manages expiration order books for a single underlying.
///
/// Provides centralized access to all expirations for an underlying asset.
/// Uses `DashMap` for thread-safe concurrent access.
pub struct ExpirationOrderBookManager {
    /// Expiration order books indexed by expiration date.
    expirations: DashMap<ExpirationDate, ExpirationOrderBook>,
    /// The underlying asset symbol.
    underlying: String,
}

impl ExpirationOrderBookManager {
    /// Creates a new expiration order book manager.
    ///
    /// # Arguments
    ///
    /// * `underlying` - The underlying asset symbol
    #[must_use]
    pub fn new(underlying: impl Into<String>) -> Self {
        Self {
            expirations: DashMap::new(),
            underlying: underlying.into(),
        }
    }

    /// Returns the underlying asset symbol.
    #[must_use]
    pub fn underlying(&self) -> &str {
        &self.underlying
    }

    /// Returns the number of expirations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.expirations.len()
    }

    /// Returns true if there are no expirations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.expirations.is_empty()
    }

    /// Gets or creates an expiration order book.
    pub fn get_or_create(
        &self,
        expiration: ExpirationDate,
    ) -> dashmap::mapref::one::Ref<'_, ExpirationDate, ExpirationOrderBook> {
        self.expirations
            .entry(expiration)
            .or_insert_with(|| ExpirationOrderBook::new(&self.underlying, expiration))
            .downgrade()
    }

    /// Gets an expiration order book.
    ///
    /// # Errors
    ///
    /// Returns `Error::ExpirationNotFound` if the expiration does not exist.
    pub fn get(
        &self,
        expiration: &ExpirationDate,
    ) -> Result<dashmap::mapref::one::Ref<'_, ExpirationDate, ExpirationOrderBook>> {
        self.expirations
            .get(expiration)
            .ok_or_else(|| Error::expiration_not_found(expiration.to_string()))
    }

    /// Returns true if an expiration exists.
    #[must_use]
    pub fn contains(&self, expiration: &ExpirationDate) -> bool {
        self.expirations.contains_key(expiration)
    }

    /// Removes an expiration order book.
    pub fn remove(&self, expiration: &ExpirationDate) -> bool {
        self.expirations.remove(expiration).is_some()
    }

    /// Returns the total order count across all expirations.
    #[must_use]
    pub fn total_order_count(&self) -> usize {
        self.expirations
            .iter()
            .map(|e| e.value().total_order_count())
            .sum()
    }

    /// Returns the total strike count across all expirations.
    #[must_use]
    pub fn total_strike_count(&self) -> usize {
        self.expirations
            .iter()
            .map(|e| e.value().strike_count())
            .sum()
    }

    /// Returns statistics about this expiration manager.
    #[must_use]
    pub fn stats(&self) -> ExpirationManagerStats {
        ExpirationManagerStats {
            underlying: self.underlying.clone(),
            expiration_count: self.len(),
            total_strikes: self.total_strike_count(),
            total_orders: self.total_order_count(),
        }
    }
}

/// Statistics about an expiration manager.
#[derive(Debug, Clone)]
pub struct ExpirationManagerStats {
    /// The underlying asset symbol.
    pub underlying: String,
    /// Number of expirations.
    pub expiration_count: usize,
    /// Total number of strikes across all expirations.
    pub total_strikes: usize,
    /// Total number of orders across all expirations.
    pub total_orders: usize,
}

impl std::fmt::Display for ExpirationManagerStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} expirations, {} strikes, {} orders",
            self.underlying, self.expiration_count, self.total_strikes, self.total_orders
        )
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
    fn test_expiration_order_book_creation() {
        let exp = ExpirationOrderBook::new("BTC", test_expiration());

        assert_eq!(exp.underlying(), "BTC");
        assert!(exp.is_empty());
    }

    #[test]
    fn test_expiration_order_book_strikes() {
        let exp = ExpirationOrderBook::new("BTC", test_expiration());

        exp.get_or_create_strike(50000);
        exp.get_or_create_strike(55000);
        exp.get_or_create_strike(45000);

        assert_eq!(exp.strike_count(), 3);
        assert_eq!(exp.strike_prices(), vec![45000, 50000, 55000]);
    }

    #[test]
    fn test_expiration_order_book_orders() {
        let exp = ExpirationOrderBook::new("BTC", test_expiration());

        let strike = exp.get_or_create_strike(50000);
        strike
            .call()
            .add_limit_order(OrderId::new(), Side::Buy, 100, 10)
            .unwrap();

        assert_eq!(exp.total_order_count(), 1);
    }

    #[test]
    fn test_expiration_manager_creation() {
        let manager = ExpirationOrderBookManager::new("BTC");

        assert!(manager.is_empty());
        assert_eq!(manager.underlying(), "BTC");
    }

    #[test]
    fn test_expiration_manager_get_or_create() {
        let manager = ExpirationOrderBookManager::new("BTC");

        manager.get_or_create(ExpirationDate::Days(pos!(30.0)));
        manager.get_or_create(ExpirationDate::Days(pos!(60.0)));
        manager.get_or_create(ExpirationDate::Days(pos!(90.0)));

        assert_eq!(manager.len(), 3);
    }
}
