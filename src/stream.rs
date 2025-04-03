use std::cell::RefCell;
use futures_signals::signal::{Signal, SignalExt, SignalStream};
use std::fmt::Write as FmtWrite;
use std::io::{stdout, Write};
use futures::stream::{ StreamExt};
use blacksholes_rust::blacksholes;
use time::{OffsetDateTime};
use tokio::time::sleep;
use std::time::Duration;
use rand::Rng;
use crate::alloc_tracker::{get_allocated};

const MIN_SECONDS: f64 = 1.0;
const MAX_SECONDS: f64 = 2.0;
const MIN_VALUE: f64 = 90.0;
const MAX_VALUE: f64 = 99.0;
const MIN_VOL: f64 = 0.1;
const MAX_VOL: f64 = 0.5;

#[derive(Debug, Clone)]
pub struct Price {
    spot: f64,
    vol: f64,
    call: f64,
    put: f64,
    time: f64
}


pub async fn start_streaming() {
    let strike_price = 100.0;
    let time_to_expiration_in_years = 1.0;
    let risk_free_rate = 0.05;


    let spot_price = create_stream(MIN_VALUE, MAX_VALUE).map(|item| { item.unwrap_or_else(|| 0.0) });
    let volatility = create_stream(MIN_VOL, MAX_VOL).map(|item| { item.unwrap_or_else(|| 0.0) });

    SignalStream::zip(spot_price.to_stream(), volatility.to_stream())
        .map(|(spot, vol)| { calculate_options(strike_price, time_to_expiration_in_years, risk_free_rate, spot, vol) })
        .scan(vec!(empty_price()), |state, price| {
            state.push(price);
            futures::future::ready(Some(state.clone()))
        })
        .for_each(|state| async move { output_price(&state); }).await;

}

pub fn calculate_options(strike_price: f64, time_to_expiration_in_years: f64, risk_free_rate: f64, spot: f64, vol: f64) -> Price {
    let call = blacksholes::calc_call(spot, strike_price, time_to_expiration_in_years, risk_free_rate, vol);
    let put = blacksholes::calc_put(spot, strike_price, time_to_expiration_in_years, risk_free_rate, vol);
    Price { spot, vol, call, put, time: OffsetDateTime::now_utc().unix_timestamp() as f64 }
}

fn empty_price() -> Price {
    Price { spot: 0.0, vol: 0.0, call: 0.0, put: 0.0, time: OffsetDateTime::now_utc().unix_timestamp() as f64}
}
thread_local! {
    static OUTPUT_BUF: RefCell<(String, Vec<u8>)> = RefCell::new((
        String::with_capacity(200),  // For formatting
        Vec::with_capacity(200)     // For raw byte output
    ));
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
            "\x1B[2J\x1B[HAllocated: {} bytes\nSpot: {:.2}\nVol: {:.2}\nCall: {:.2}\nPut: {:.2}",
            get_allocated(),
            price.spot,
            price.vol,
            price.call,
            price.put
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

