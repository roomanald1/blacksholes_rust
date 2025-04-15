use std::cell::RefCell;
use futures_signals::signal::{SignalExt};
use std::fmt::Write as FmtWrite;
use std::io;
use std::io::{stdout, Read, Write};
use chrono::NaiveDate;
use crate::data::btc_fetch::{btc};
use crate::data::btc_fetch::btc::fetch_live_option_price;
use crate::utils::date_utils::round_to_nearest_1000;
use crate::utils::OptionType;
use crate::pricing::option_pricer::{OptionPricer, OptionValue};
use crate::pricing::volatility::TimeAwareEwmaVolatility;

#[derive(Debug, Clone)]
pub struct Price {
    spot: Option<f64>,
    strike: Option<f64>,
    call: OptionValue,
    put: OptionValue,
    monte_c_premium: Option<f64>,
    monte_p_premium: Option<f64>,
}

pub async fn start_streaming() -> Result<(), String> {
    let expiry = NaiveDate::from_ymd(2025, 6, 27);
    let time_to_expiration_in_years = crate::utils::date_utils::time_to_expiry_in_years(expiry);

    let hist = btc::fetch_prev_ndays(90).await.map_err(|e| e.to_string())?;
    let mut vol = TimeAwareEwmaVolatility::new(0.94, 30);

    //Initialise Weighted Moving Average Vol Calc
    hist.iter().for_each(|i| {
        vol.update(i);
    });
    Ok(())
    /*
    let spot_price = btc::create_1s_ticking(1).await;
    spot_price
        .map(|opt|  {
            opt.map(|price| {
                let vol = vol.update(&price).unwrap_or(0.0);
                (price, vol)
            })
        })
        .for_each(|opt| async move {
            if let Some((spot, vol)) = opt {

                let nearest_1000 = round_to_nearest_1000(spot.close as i32);
                let strikes = (-5..5).step_by(1).map(|i| nearest_1000 as f64 + (i as f64 * 1000.0)).collect::<Vec<f64>>();
                let risk_free_rate = 0.04;
                let prices = strikes.into_iter().map(|strike_price | {
                    calculate_options(strike_price, time_to_expiration_in_years, risk_free_rate, spot.close, vol)
                }).collect::<Vec<Price>>();

                output_price(&prices, risk_free_rate, vol, time_to_expiration_in_years);
            }

        }).await;

    Ok(())*/
}

pub fn calculate_options(strike_price: f64, time_to_expiration_in_years: f64, risk_free_rate: f64, spot: f64, vol: f64) -> Price {

    let mut option_pricer = OptionPricer::new(spot, strike_price, time_to_expiration_in_years, risk_free_rate, vol);
    let call = option_pricer.calculate_option(OptionType::Call);
    //let monte_price_c = monte_carlo(OptionType::Call, spot, risk_free_rate, vol, time_to_expiration_in_years, strike_price, 1_000_000);

    let put = option_pricer.calculate_option(OptionType::Put);
    //let monte_price_p = monte_carlo(OptionType::Put, spot, risk_free_rate, vol, time_to_expiration_in_years, strike_price, 1_000_000);

    Price {
        spot: Some(spot),
        call, put,
        strike: Some(strike_price),
        monte_p_premium: None,//Some(monte_price_p),
        monte_c_premium: None//Some(monte_price_c)
    }
}

thread_local! {
    static OUTPUT_BUF: RefCell<(String, Vec<u8>)> = RefCell::new((
        String::with_capacity(200),  // For formatting
        Vec::with_capacity(200)     // For raw byte output
    ));
}

fn output_price(prices: &[Price], risk_free_rate: f64, volatility: f64, time_to_expiration_in_years: f64) {

    OUTPUT_BUF.with(|buf| {
        let mut buffer = buf.borrow_mut();
        let (string_buf, byte_buf) = &mut *buffer;

        let price = prices.first().unwrap();
        write!(
            string_buf,
            "\x1B[2J\x1B[\
                HSpot: {:.2} Risk Free Rate: {:.2} Volatility {:.2} Expiry {:.2}",
            price.spot.unwrap_or(f64::NAN),
            risk_free_rate,
            volatility,
            time_to_expiration_in_years
        ).unwrap();


        prices.iter().for_each(|price| {
            write!(
                string_buf,
                "\x1B[2J\x1B[\
                HStrike: {:.2} => || Call [ Premium (BS): {:.2} Delta: {:.2} Gamma: {:.2} Vega: {:.2} Theta: {:.2} Rho: {:.2}] || Put  [ Premium (BS): {:.2} Delta: {:.2} Gamma: {:.2} Vega: {:.2} Theta: {:.2} Rho: {:.2}] ||",
                price.strike.unwrap_or(f64::NAN),
                price.call.premium.unwrap_or(f64::NAN),
                //price.monte_c_premium.unwrap_or(f64::NAN),
                price.call.delta.unwrap_or(f64::NAN),
                price.call.gamma.unwrap_or(f64::NAN),
                price.call.vega.unwrap_or(f64::NAN),
                price.call.theta.unwrap_or(f64::NAN),
                price.call.rho.unwrap_or(f64::NAN),
                price.put.premium.unwrap_or(f64::NAN),
                //price.monte_p_premium.unwrap_or(f64::NAN),
                price.put.delta.unwrap_or(f64::NAN),
                price.put.gamma.unwrap_or(f64::NAN),
                price.put.vega.unwrap_or(f64::NAN),
                price.put.theta.unwrap_or(f64::NAN),
                price.put.rho.unwrap_or(f64::NAN)
            ).unwrap();
        });
        // Build complete output in one pass


        // Convert to bytes and write
        byte_buf.extend_from_slice(string_buf.as_bytes());
        let mut handle = stdout().lock();
        handle.write_all(&byte_buf).unwrap();//lock gets raw stdout
        handle.flush().unwrap();
        string_buf.clear();
        byte_buf.clear();
    })

}