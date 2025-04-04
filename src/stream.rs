use std::cell::RefCell;
use futures_signals::signal::{Signal, SignalExt, SignalStream};
use std::fmt::Write as FmtWrite;
use std::io::{stdout, Write};
use futures::stream::{ StreamExt};
use time::{OffsetDateTime};
use tokio::time::sleep;
use std::time::Duration;
use rand::Rng;
use blacksholes_rust::blacksholes::{OptionPricer, OptionType, OptionValue};
use crate::price_history::PriceHistory;
use std::borrow::BorrowMut;

const MIN_SECONDS: f64 = 1.0;
const MAX_SECONDS: f64 = 2.0;
const MIN_VALUE: f64 = 90.0;
const MAX_VALUE: f64 = 99.0;
const MIN_VOL: f64 = 0.1;
const MAX_VOL: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct Price {
    spot: Option<f64>,
    strike: Option<f64>,
    vol: Option<f64>,
    call: OptionValue,
    put: OptionValue,
    time: Option<f64>,
}



pub async fn start_streaming() {
    let strike_price = 100.0;
    let time_to_expiration_in_years = 1.0;
    let risk_free_rate = 0.05;


    let spot_price = create_stream(MIN_VALUE, MAX_VALUE).map(|item| { item.unwrap_or_else(|| 0.0) });
    let volatility = create_stream(MIN_VOL, MAX_VOL).map(|item| { item.unwrap_or_else(|| 0.0) });

    SignalStream::zip(spot_price.to_stream(), volatility.to_stream())
        .map(|(spot, vol)| { calculate_options(strike_price, time_to_expiration_in_years, risk_free_rate, spot, vol) })
        .for_each(|price| async move {
            PRICE_HISTORY.with(|price_history| {
                price_history.borrow_mut().add_price(price);
                output_price(price_history.borrow_mut().latest_prices());
            })
        }).await;

}

pub fn calculate_options(strike_price: f64, time_to_expiration_in_years: f64, risk_free_rate: f64, spot: f64, vol: f64) -> Price {

    let mut option_pricer = OptionPricer::new(spot, strike_price, time_to_expiration_in_years, risk_free_rate, vol);
    let call = option_pricer.calculate_option(OptionType::Call);
    let put = option_pricer.calculate_option(OptionType::Put);
    Price {
        spot: Some(spot),
        vol: Some(vol),
        call, put,
        time: Some(OffsetDateTime::now_utc().unix_timestamp() as f64) ,
        strike: Some(strike_price)
    }
}


fn empty_price() -> Price {
    Price { spot: None, vol: None, call: empty_option_price(), put: empty_option_price(), time: None, strike: None}
}
fn empty_option_price() -> OptionValue {
    OptionValue { option_type: None, premium: None, delta: None, gamma: None, vega: None, theta: None, rho: None}
}

thread_local! {
    static OUTPUT_BUF: RefCell<(String, Vec<u8>)> = RefCell::new((
        String::with_capacity(200),  // For formatting
        Vec::with_capacity(200)     // For raw byte output
    ));
    static PRICE_HISTORY: RefCell<PriceHistory> = RefCell::new(PriceHistory::new(10));
}

fn output_price(prices: &[Price]) {

    OUTPUT_BUF.with(|buf| {
        let mut buffer = buf.borrow_mut();
        let (string_buf, byte_buf) = &mut *buffer;

        let empty_price = empty_price();
        let price = prices.last().unwrap_or(&empty_price);

        // Build complete output in one pass
        write!(
            string_buf,
            "\x1B[2J\x1B[\
            HSpot: {:.2}\n\
            Strike: {:.2}\n\
            Vol: {:.2}\n\
            History: {:.2}\n\
            Call [ Premium: {:.2} Delta: {:.2} Gamma: {:.2} Vega: {:.2} Theta: {:.2} Rho: {:.2}]\n\
            Put  [ Premium: {:.2} Delta: {:.2} Gamma: {:.2} Vega: {:.2} Theta: {:.2} Rho: {:.2}]",
            price.spot.unwrap_or(f64::NAN),
            price.strike.unwrap_or(f64::NAN),
            price.vol.unwrap_or(f64::NAN),
            prices.len(),
            price.call.premium.unwrap_or(f64::NAN),
            price.call.delta.unwrap_or(f64::NAN),
            price.call.gamma.unwrap_or(f64::NAN),
            price.call.vega.unwrap_or(f64::NAN),
            price.call.theta.unwrap_or(f64::NAN),
            price.call.rho.unwrap_or(f64::NAN),
            price.put.premium.unwrap_or(f64::NAN),
            price.put.delta.unwrap_or(f64::NAN),
            price.put.gamma.unwrap_or(f64::NAN),
            price.put.vega.unwrap_or(f64::NAN),
            price.put.theta.unwrap_or(f64::NAN),
            price.put.rho.unwrap_or(f64::NAN)
        ).unwrap();

        // Convert to bytes and write
        byte_buf.extend_from_slice(string_buf.as_bytes());
        let mut handle = stdout().lock();
        handle.write_all(&byte_buf).unwrap();//lock gets raw stdout
        handle.flush().unwrap();
        string_buf.clear();
        byte_buf.clear();
    })

}


fn create_stream(min: f64, max: f64) -> impl Signal<Item = Option<f64>> {
    let mut rng = rand::rng();
    futures_signals::signal::from_stream(async_stream::stream! {
        loop {
            let delay = rng.random_range(MIN_SECONDS..=MAX_SECONDS);
            let value = rng.random_range(min..=max);

            sleep(Duration::from_secs_f64(delay)).await;
            yield value;
        }
    })
}

