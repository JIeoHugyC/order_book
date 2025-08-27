pub mod types;

use std::collections::{BTreeMap, VecDeque};
use types::{Order, Side, Trade, Price, Quantity};
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct OrderBook {
    /// Bids: higher prices first (reverse order)
    bids: BTreeMap<Price, VecDeque<Order>>,
    /// Asks: lower prices first (natural order)
    asks: BTreeMap<Price, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn place_order(&mut self, side: Side, price: i32, quantity: i32) -> Vec<Trade> {
        let mut incoming_order = Order::new(Uuid::new_v4(), side, price.into(), quantity.into());
        let mut trades = Vec::new();

        match side {
            Side::Buy => {
                // Match against asks (sell orders)
                self.match_order(&mut incoming_order, &mut trades, true);
                // Add remainder to bids if any quantity left
                if *incoming_order.quantity > 0 {
                    self.add_order_to_book(incoming_order);
                }
            }
            Side::Sell => {
                // Match against bids (buy orders)
                self.match_order(&mut incoming_order, &mut trades, false);
                // Add remainder to asks if any quantity left
                if *incoming_order.quantity > 0 {
                    self.add_order_to_book(incoming_order);
                }
            }
        }

        trades
    }

    fn match_order(&mut self, incoming_order: &mut Order, trades: &mut Vec<Trade>, matching_against_asks: bool) {
        let opposite_book = if matching_against_asks {
            &mut self.asks
        } else {
            &mut self.bids
        };

        let mut prices_to_remove = Vec::new();

        // Get price levels in correct order
        let price_levels: Vec<Price> = if matching_against_asks {
            // For buy orders matching against asks: start from lowest ask price
            opposite_book.keys().cloned().collect()
        } else {
            // For sell orders matching against bids: start from highest bid price
            opposite_book.keys().cloned().collect::<Vec<_>>().into_iter().rev().collect()
        };

        for price_level in price_levels {
            if *incoming_order.quantity == 0 {
                break;
            }

            // Check if we can match at this price level
            let can_match = if matching_against_asks {
                incoming_order.price >= price_level
            } else {
                incoming_order.price <= price_level
            };

            if !can_match {
                break;
            }

            if let Some(order_queue) = opposite_book.get_mut(&price_level) {
                while let Some(resting_order) = order_queue.front_mut() {
                    if *incoming_order.quantity == 0 {
                        break;
                    }

                    let trade_quantity = (*incoming_order.quantity).min(*resting_order.quantity);

                    let trade = Trade::new(
                        price_level.into(),
                        trade_quantity.into(),
                        resting_order.id,
                        incoming_order.id,
                    );
                    trades.push(trade);

                    incoming_order.quantity = (*incoming_order.quantity - trade_quantity).into();
                    resting_order.quantity = (*resting_order.quantity - trade_quantity).into();

                    if *resting_order.quantity == 0 {
                        order_queue.pop_front();
                    }
                }

                if order_queue.is_empty() {
                    prices_to_remove.push(price_level);
                }
            }
        }

        // Clean up empty price levels
        for price in prices_to_remove {
            opposite_book.remove(&price);
        }
    }

    fn add_order_to_book(&mut self, order: Order) {
        let book = match order.side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };

        book.entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn best_buy(&self) -> Option<(Price, Quantity)> {
        // Highest bid price (last in BTreeMap)
        self.bids.last_key_value().map(|(price, orders)| {
            ((*price).into(), aggregate_quantity_at_price(orders))
        })
    }

    pub fn best_sell(&self) -> Option<(Price, Quantity)> {
        // Lowest ask price (first in BTreeMap)
        self.asks.first_key_value().map(|(price, orders)| {
            ((*price).into(), aggregate_quantity_at_price(orders))
        })
    }
}

fn aggregate_quantity_at_price(orders: &VecDeque<Order>) -> Quantity {
    let total: i32 = orders.iter().map(|order| *order.quantity).sum();
    total.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_order_book() {
        let book = OrderBook::new();
        assert_eq!(book.best_buy(), None);
        assert_eq!(book.best_sell(), None);
    }

    #[test]
    fn test_simple_buy_sell_match() {
        let mut book = OrderBook::new();

        // Add sell order first
        let trades = book.place_order(Side::Sell, 100, 50);
        assert!(trades.is_empty());

        // Add matching buy order
        let trades = book.place_order(Side::Buy, 100, 30);
        assert_eq!(trades.len(), 1);

        let trade = &trades[0];
        assert_eq!(*trade.price, 100);
        assert_eq!(*trade.quantity, 30);
        assert_ne!(trade.maker_id, trade.taker_id);

        // Check remaining sell order
        assert_eq!(book.best_sell(), Some((Price(100), Quantity(20))));
    }

    #[test]
    fn test_partial_fill_and_remainder() {
        let mut book = OrderBook::new();

        // Add sell order
        book.place_order(Side::Sell, 100, 30);

        // Add larger buy order
        let trades = book.place_order(Side::Buy, 105, 50);
        assert_eq!(trades.len(), 1);

        let trade = &trades[0];
        assert_eq!(*trade.price, 100); // Trade at resting order price
        assert_eq!(*trade.quantity, 30);

        // Check remainder buy order was added
        assert_eq!(book.best_buy(), Some((Price(105), Quantity(20))));
        assert_eq!(book.best_sell(), None);
    }

    #[test]
    fn test_price_time_priority() {
        let mut book = OrderBook::new();

        // Add multiple sell orders at same price
        book.place_order(Side::Sell, 100, 10); // First (oldest)
        book.place_order(Side::Sell, 100, 20); // Second
        book.place_order(Side::Sell, 100, 15); // Third

        // Buy order that matches all
        let trades = book.place_order(Side::Buy, 100, 45);
        assert_eq!(trades.len(), 3);

        // Check time priority - oldest order first
        assert_eq!(*trades[0].quantity, 10);
        assert_eq!(*trades[1].quantity, 20);
        assert_eq!(*trades[2].quantity, 15);
    }

    #[test]
    fn test_price_priority() {
        let mut book = OrderBook::new();

        // Add buy orders at different prices
        book.place_order(Side::Buy, 98, 10);   // Lower price
        book.place_order(Side::Buy, 102, 15);  // Higher price (best)
        book.place_order(Side::Buy, 100, 20);  // Middle price

        // Sell order matches with highest bid first
        let trades = book.place_order(Side::Sell, 98, 50);
        assert_eq!(trades.len(), 3);

        // Check price priority - highest bid first
        assert_eq!(*trades[0].price, 102);
        assert_eq!(*trades[1].price, 100);
        assert_eq!(*trades[2].price, 98);
    }

    #[test]
    fn test_no_match_different_prices() {
        let mut book = OrderBook::new();

        // Add sell order at high price
        book.place_order(Side::Sell, 105, 10);

        // Add buy order at lower price - no match
        let trades = book.place_order(Side::Buy, 95, 10);
        assert!(trades.is_empty());

        // Both orders should remain in book
        assert_eq!(book.best_buy(), Some((Price(95), Quantity(10))));
        assert_eq!(book.best_sell(), Some((Price(105), Quantity(10))));
    }

    #[test]
    fn test_multiple_price_levels() {
        let mut book = OrderBook::new();

        // Build order book with multiple levels
        book.place_order(Side::Sell, 101, 10);
        book.place_order(Side::Sell, 102, 15);
        book.place_order(Side::Sell, 103, 20);

        book.place_order(Side::Buy, 99, 10);
        book.place_order(Side::Buy, 98, 15);

        // Large buy order crosses spread
        let trades = book.place_order(Side::Buy, 102, 30);
        assert_eq!(trades.len(), 2);

        // Should match 101 level completely, then completely match 102 level
        assert_eq!(*trades[0].price, 101);
        assert_eq!(*trades[0].quantity, 10);
        assert_eq!(*trades[1].price, 102);
        assert_eq!(*trades[1].quantity, 15);

        // Check remaining book state
        assert_eq!(book.best_sell(), Some((Price(103), Quantity(20)))); // 103 level remains untouched
        assert_eq!(book.best_buy(), Some((Price(102), Quantity(5)))); // remainder of incoming buy order
    }

    #[test]
    fn test_best_buy_sell_aggregation() {
        let mut book = OrderBook::new();

        // Add multiple orders at same price level
        book.place_order(Side::Buy, 100, 10);
        book.place_order(Side::Buy, 100, 20);
        book.place_order(Side::Buy, 100, 15);

        book.place_order(Side::Sell, 105, 25);
        book.place_order(Side::Sell, 105, 30);

        // Check aggregated quantities
        assert_eq!(book.best_buy(), Some((Price(100), Quantity(45)))); // 10 + 20 + 15
        assert_eq!(book.best_sell(), Some((Price(105), Quantity(55)))); // 25 + 30
    }

    #[test]
    fn test_realistic_trading_scenario() {
        let mut book = OrderBook::new();

        // Build realistic order book
        // Sell side
        book.place_order(Side::Sell, 105, 100);
        book.place_order(Side::Sell, 104, 200);
        book.place_order(Side::Sell, 103, 150);

        // Buy side
        book.place_order(Side::Buy, 102, 180);
        book.place_order(Side::Buy, 101, 220);
        book.place_order(Side::Buy, 100, 300);

        // Spread should be 102 bid, 103 ask
        assert_eq!(book.best_buy(), Some((Price(102), Quantity(180))));
        assert_eq!(book.best_sell(), Some((Price(103), Quantity(150))));

        // Large market order crosses spread
        let trades = book.place_order(Side::Buy, 106, 500);

        // Should execute against all ask levels
        assert_eq!(trades.len(), 3);

        let total_traded: i32 = trades.iter().map(|t| *t.quantity).sum();
        assert_eq!(total_traded, 450); // 150 + 200 + 100

        // Check final state - buy order remainder should be in book
        assert_eq!(book.best_buy(), Some((Price(106), Quantity(50)))); // 500 - 450 = 50 remaining
    }
}