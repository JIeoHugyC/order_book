use std::ops::Deref;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Price(pub i32);

impl Deref for Price {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for Price {
    fn from(value: i32) -> Self {
        Price(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quantity(pub i32);

impl Deref for Quantity {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<i32> for Quantity {
    fn from(value: i32) -> Self {
        Quantity(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    pub id: Uuid,
    pub side: Side,
    pub price: Price,
    pub quantity: Quantity,
}

impl Order {
    pub fn new(id: Uuid, side: Side, price: Price, quantity: Quantity) -> Self {
        Order {
            id,
            side,
            price,
            quantity,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trade {
    pub price: Price,
    pub quantity: Quantity,
    pub maker_id: Uuid,
    pub taker_id: Uuid,
}

impl Trade {
    pub fn new(price: Price, quantity: Quantity, maker_id: Uuid, taker_id: Uuid) -> Self {
        Trade {
            price,
            quantity,
            maker_id,
            taker_id,
        }
    }
}