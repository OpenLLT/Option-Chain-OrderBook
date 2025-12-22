//! Integration tests for the orderbook module.

use option_chain_orderbook::orderbook::{OptionOrderBook, OptionOrderBookManager};
use orderbook_rs::{OrderId, Side};

#[test]
fn test_option_order_book_integration() {
    let book = OptionOrderBook::new("BTC-20240329-50000-C");

    // Add orders
    book.add_limit_order(OrderId::new(), Side::Buy, 100, 10)
        .unwrap();
    book.add_limit_order(OrderId::new(), Side::Sell, 101, 5)
        .unwrap();

    // Verify state
    assert_eq!(book.order_count(), 2);
    assert!(book.best_quote().is_two_sided());
}

#[test]
fn test_manager_integration() {
    let mut manager = OptionOrderBookManager::new();

    // Create multiple books
    let book1 = manager.get_or_create("BTC-20240329-50000-C");
    book1
        .add_limit_order(OrderId::new(), Side::Buy, 100, 10)
        .unwrap();

    let book2 = manager.get_or_create("BTC-20240329-50000-P");
    book2
        .add_limit_order(OrderId::new(), Side::Sell, 50, 5)
        .unwrap();

    // Verify aggregation
    assert_eq!(manager.len(), 2);
    assert_eq!(manager.total_order_count(), 2);
}
