use std::collections::VecDeque;
use crate::stream::Price;

pub struct PriceHistory {
    capacity: usize,
    buffer: VecDeque<Price>,
}

impl PriceHistory {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            buffer: VecDeque::with_capacity(capacity),
        }
    }

    pub fn add_price(&mut self, price: Price) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front(); // Remove oldest
        }
        self.buffer.push_back(price);
    }

    pub fn latest_prices(&mut self) -> &[Price] {
        self.buffer.make_contiguous()
    }
}