use std::sync::Arc;
use blacksholes_rust::book::book::{start_market_maker_delta_neutral_strategy_loop, Book, Market, MarketExecutor, MarketMakerParams};
use blacksholes_rust::book::contract::OptionContract;

#[tokio::main]
async fn main() -> Result<(), String> {

    let executor = MarketExecutor::new();
    let book = Book::new(executor);
    let market = Market::new(vec![
        OptionContract::new("27JUN25".to_string(), 80000.0)
    ]);

    Market::start(Arc::clone(&market));

    start_market_maker_delta_neutral_strategy_loop(
        market,
        book,
        MarketMakerParams {
            initial_pnl: 0.0,
            delta_threshold: 0.01,
            desired_spread_percentage: 0.1,
            max_exposure: 1.0,
            gamma_scalp_threshold: 1.0,
        }).await.map_err(|e| e.to_string())?;

    Ok(())
}