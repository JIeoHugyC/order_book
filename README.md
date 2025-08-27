# Order Book

A simple in-memory limit order book implementation in Rust.

## What it does

This order book matches buy and sell orders using price-time priority:
- Better prices execute first
- At the same price, older orders execute first
- Orders can be partially filled
- Trade price equals the resting order's price

## API

```rust
// Place an order
let trades = book.place_order(Side::Buy, 100, 50);

// Get best prices
let best_bid = book.best_buy();    // Some((Price(99), Quantity(100)))
let best_ask = book.best_sell();   // Some((Price(101), Quantity(200)))
```

## How to test

Run all tests:
```bash
cargo test
```

## Example

```rust
use order_book::{OrderBook, types::Side};

let mut book = OrderBook::new();

// Add sell order
book.place_order(Side::Sell, 100, 50);

// Add buy order that matches
let trades = book.place_order(Side::Buy, 100, 30);

// One trade executed: 30 shares at price 100
assert_eq!(trades.len(), 1);
assert_eq!(*trades[0].quantity, 30);
```

## How it works

- Uses `BTreeMap` for automatic price sorting
- Uses `VecDeque` for time priority within price levels
- Buy orders match against sell orders (asks)
- Sell orders match against buy orders (bids)
- Remaining quantity gets added to the book