use std::sync::{Arc};
use tokio::sync::Mutex;
use std::thread;
use std::time::Duration;
use chrono::NaiveDate;
use rand_distr::num_traits::Signed;
use crate::book::contract::OptionContract;
use crate::pricing::option_pricer::{OptionPricer, OptionValue};
use crate::utils;
use crate::utils::OptionType;


pub enum Direction {
    Buy, Sell
}

#[derive(Clone)]
struct OptionTrade {
    price: f64,
    option_type: OptionType,
    amount: f64,
    contract_expiry: NaiveDate
}

pub struct MarketExecutor {

}

impl MarketExecutor {
    pub fn new() -> MarketExecutor {
        MarketExecutor {}
    }

    pub fn execute_spot(&self, amount:f64) -> Result<(), String> {
        Ok(())
    }

    pub fn execute_future_perp(&self, amount:f64, direction: Direction) -> Result<(), String> {
        //BTC-PERP
        Ok(())
    }
    pub fn execute_future(&self, amount:f64, strike: f64, expiry: String, direction: Direction) -> Result<(), String> {
        Ok(())
    }

    pub fn execute_option(&self, amount:f64, strike: f64, expiry: String, direction: Direction, option_type: OptionType) -> Result<(), String> {
        Ok(())
    }
}

pub struct Book {
    executed:Vec<OptionTrade>,
    positions: Vec<OptionTrade>,
    delta_position: f64,
    gamma_position: f64,
    pnl: f64,
    futures_hedge: f64,
    market_executor: MarketExecutor,
}

impl Book {
    pub fn new(market_executor: MarketExecutor) -> Book {
        Book {
            positions: vec![],
            executed: vec![],
            delta_position: 0.0,
            pnl: 0.0,
            futures_hedge: 0.0,
            gamma_position: 0.0,
            market_executor
        }
    }

    pub fn update_greeks(&mut self, delta: f64, gamma: f64){
        self.delta_position = delta;
        self.gamma_position = gamma;
    }

}

pub struct MarketMakerParams {
    pub initial_pnl: f64,
    pub desired_spread_percentage: f64,
    pub max_exposure: f64,
    pub delta_threshold: f64,
    pub gamma_scalp_threshold: f64
}


fn should_use_futures(market: &Market) -> bool {
    // Simple heuristic: use futures when funding rate is favorable
    market.funding_rate.abs() < 0.0001 // 0.01%
}

pub fn calculate_book_risk(market: &Market, book: &Book) -> Result<Vec<OptionValue>, String>{
    let mut book_risk: Vec<OptionValue> = vec![];

    //Evaluate Existing Positions
    for trade in book.positions.clone()
    {
        let contract_md= market.contracts
            .iter()
            .find(|c| c.expiry == trade.contract_expiry.format("y%m%d%").to_string())
            .map(|c| if trade.option_type == c.call.option_type {&c.call}else {&c.put})
            .ok_or("No matching market data for trade contract")?;

        let mut pricer = OptionPricer::new(
            contract_md.latest_index_price.unwrap(),
            trade.price,
            utils::date_utils::time_to_expiry_in_years(trade.contract_expiry),
            0.0,
            contract_md.mark_iv.unwrap());

        let result = pricer.calculate_option(trade.option_type);
        book_risk.push(result);
    }

    println!("Book Risks {:?}", book_risk);
    Ok(book_risk)
}


pub fn delta_balance_book(book: &mut Book, market: &Market) -> Result<(), String> {
    let delta_diff = book.delta_position - book.futures_hedge;

    if delta_diff.abs() > 0.01 { // Avoid micro-adjustments
        // Choose hedging instrument based on market conditions
        let use_futures = should_use_futures(market);

        if use_futures {
            // Futures hedging logic
            let hedge_amount = delta_diff;
            book.market_executor.execute_future_perp(hedge_amount, Direction::Buy)?;
            println!("Adjusting futures position by {:.2} BTC", hedge_amount);
            book.futures_hedge += hedge_amount;

            // Calculate and deduct costs
            let fee = hedge_amount.abs() * market.futures_fee;
            book.pnl -= fee;
        } else {
            // Spot hedging logic
            println!("Adjusting spot position by {:.2} BTC", delta_diff);
        }
    }

    Ok(())
}


pub fn gamma_scalp(book: &mut Book, new_delta: f64, threshold: f64) -> Result<(), String> {
    let delta_diff = (book.delta_position - new_delta).abs();

    if delta_diff > threshold {
        let adjustment = new_delta - book.delta_position;
        println!("Gamma scalp: adjusting delta by {:.4}", adjustment);

        if adjustment > 0.0 {
            // Market rose → Options became more negative delta
            // Need to BUY more underlying (futures/spot)
            book.market_executor.execute_future_perp(adjustment.abs(), Direction::Buy)?;
            book.futures_hedge += adjustment;
        } else {
            // Market fell → Options became less negative delta
            // Need to SELL excess underlying
            book.market_executor.execute_future_perp(adjustment.abs(), Direction::Sell)?;
            book.futures_hedge += adjustment;
        }

        book.delta_position = new_delta;
    }
    Ok(())
}

pub async fn start_market_maker_delta_neutral_strategy_loop(
    market: Arc<Mutex<Market>>,
    mut book: Book,
    params: MarketMakerParams,
) -> Result<(), String> {

    loop {
        let market = market.lock().await;
        let book_risk = calculate_book_risk(&market, &book)?;

        let total_delta: f64 = book_risk.iter()
            .filter_map(|r| r.delta)
            .sum();

        let total_gamma: f64 = book_risk.iter()
            .filter_map(|r| r.gamma)
            .sum();

        book.update_greeks(total_delta, total_gamma);

        // Gamma scalping
        gamma_scalp(&mut book, total_delta, params.gamma_scalp_threshold)?;

        // Delta hedging
        delta_balance_book(&mut book, &market)?;

        // Trading logic
        if book.pnl.abs() < params.max_exposure {
            // Implement your market making logic here
            // - Find best bid/ask spreads
            // - Place new orders
            // - Manage inventory
        }

        thread::sleep(Duration::from_secs(60 * 10)); // Re-evaluate every 10 minutes
    }
}





















pub struct Market {
    contracts: Vec<OptionContract>,
    funding_rate: f64,
    futures_fee: f64,
}

impl Market {
    pub fn new(contracts: Vec<OptionContract>) -> Arc<Mutex<Market>> {
        Arc::new(Mutex::new(Market { contracts, funding_rate: 0.0, futures_fee: 0.0 }))
    }

    pub fn start(market: Arc<Mutex<Self>>) {
        tokio::spawn(async move {
            loop {
                let mut market = market.lock().await;
                for mut contract in &mut market.contracts {
                    match contract.eval().await {
                        Ok(_) => {
                            println!("Call {:?}", contract.call);
                            println!("Put {:?}", contract.call);
                        },
                        Err(e) => {
                            println!("error: {}", e);
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(1)).await
            }
        });
    }
}


