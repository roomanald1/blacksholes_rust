// src/lib.rs
pub mod option_pricer;
pub mod monte_carlo;
pub mod stream;
pub mod price_history;

pub mod utils {
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum OptionType {
        Call, Put
    }
}


