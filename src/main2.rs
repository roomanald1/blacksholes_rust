use std::cmp::Ordering;
use std::io;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use chrono::NaiveDate;
use dhat::Profiler;
use blacksholes_rust::data::btc_fetch::btc::fetch_live_option_price;
use blacksholes_rust::pricing::option_pricer::OptionPricer;
use blacksholes_rust::{stream, utils};
use blacksholes_rust::book::trading_book;
use blacksholes_rust::data::btc_fetch::btc;
use blacksholes_rust::stream::start_streaming;
use blacksholes_rust::utils::OptionType;


/*
#[tokio::main]
async fn main()  {

    trading_book::test().await.unwrap();

   /* let expiry = NaiveDate::from_ymd(2025, 6, 27);
    let strike = 80_000.0;
    let risk_free_rate = 0.0;
    let result = fetch_live_option_price(strike, format!("{}",expiry.format("%d%b%y")), OptionType::Call).await;
    match result.clone() {
        Ok(response) => {
            println!("Fetch Live Option Price response {:?}", response);


            let mut p = OptionPricer::new(response.result.index_price, strike, utils::date_utils::time_to_expiry_in_years(expiry), risk_free_rate, response.result.mark_iv / 100.0);

            let calculated_result = p.calculate_option(OptionType::Call);

            println!("Calculated Option : {:?} Greeks from EXCH {:?}", calculated_result, response.result.greeks);
            let mut bids = get_bids(response.clone(), calculated_result.premium.unwrap());
            bids.sort_by(|a, b| a.spread.partial_cmp(&b.spread).unwrap_or(std::cmp::Ordering::Equal));
            println!("Bids : {:?}", bids);

            let mut asks = get_asks(response.clone(), calculated_result.premium.unwrap());
            asks.sort_by(|a, b| a.spread.partial_cmp(&b.spread).unwrap_or(std::cmp::Ordering::Equal));
            asks.iter().for_each(|ask| {
                let mut p = OptionPricer::new(response.result.index_price, ask.price, utils::date_utils::time_to_expiry_in_years(expiry), risk_free_rate, response.result.mark_iv / 100.0);

                let calculated_result = p.calculate_option(OptionType::Call);
                println!("{:?} {:?}", ask, calculated_result)
            })
        },
        Err(e) => {
            println!("Failed to fetch Live Option Price: {}", e);
        }
    }

*/
    io::stdin().read(&mut [0u8]).unwrap();



    disable_stdout_buffering().unwrap();
    let _dhat = Profiler::new_heap();
    let ctrl_c = tokio::signal::ctrl_c();
    tokio::select! {
        _ = start_streaming() => {
            //println!("Streaming completed");
        },
        _ = ctrl_c => {
            //println!("Ctrl+C");
        },
    }
    //Comment out above and just run below to get dhat memory output
    //Also need to uncomment line from alloc_tracker
    //profile_blacksholes()
}

 */
fn disable_stdout_buffering() -> io::Result<()> {
    // Flush any existing output first
    io::stdout().flush()?;

    unsafe {
        // Get stdout file descriptor
        let stdout_fd = io::stdout().as_raw_fd();

        // Open as FILE* for setvbuf
        let stdout = libc::fdopen(stdout_fd, b"w\0".as_ptr() as *const libc::c_char);
        if stdout.is_null() {
            return Err(io::Error::last_os_error());
        }

        // Disable all buffering
        if libc::setvbuf(stdout, std::ptr::null_mut(), libc::_IONBF, 0) != 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}



fn profile_blacksholes(){
    stream::calculate_options(100.0, 1.0, 0.05, 98.0, 0.2);
}



#[derive(Clone, Copy, Debug)]
struct MarketData {
    timestamp: f64,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}
/*
// Time-weighted Parkinson volatility
fn time_weighted_parkinson_volatility(data: &[MarketData]) -> f64 {
    if data.len() < 2 {
        println!("Insufficient data: {} intervals", data.len());
        return 0.0;
    }

    let mut volatilities = Vec::new();
    let mut total_time = 0.0;
    let mut valid_intervals = 0;
    let mut hl_ranges = Vec::new();

    for i in 0..data.len() - 1 {
        let current = data[i];
        let next = data[i + 1];

        let duration = (next.timestamp - current.timestamp) / 3_600_000_000_000.0; // µs to hours
        if duration <= 0.0 || duration > 2.0 {
            println!("Invalid duration at index {}: {} hours (ts: {} to {})", i, duration, current.timestamp, next.timestamp);
            continue;
        }

        if current.high > current.low && current.high.is_finite() && current.low.is_finite() {
            let hl_ratio = (current.high - current.low) / current.low * 100.0;
            let log_hl = (current.high / current.low).ln();
            let volatility = (log_hl * log_hl / (4.0 * 2.0_f64.ln())).sqrt();
            if volatility.is_finite() {
                volatilities.push((volatility, duration));
                total_time += duration;
                valid_intervals += 1;
                hl_ranges.push(hl_ratio);
            }
        }
    }

    if volatilities.is_empty() {
        println!("No valid intervals after filtering high=low");
        return 0.0;
    }

    let weighted_vol = volatilities
        .iter()
        .map(|&(vol, duration)| vol * duration)
        .sum::<f64>()
        / total_time;

    let hours_per_year = 8_760.0;
    let annualization_factor = (hours_per_year / total_time).sqrt();

    let avg_hl = if !hl_ranges.is_empty() {
        hl_ranges.iter().sum::<f64>() / hl_ranges.len() as f64
    } else {
        0.0
    };

    println!("Raw Parkinson Vol: {:.8}", weighted_vol);
    println!("Valid Intervals: {} ({:.2}%)", valid_intervals, valid_intervals as f64 / data.len() as f64 * 100.0);
    println!("Total Time (hours): {}", total_time);
    println!("Annualization Factor: {:.4}", annualization_factor);
    println!("Average High-Low Range: {:.6}%", avg_hl);

    weighted_vol * annualization_factor
}

// Time-weighted Garman-Klass volatility
fn time_weighted_garman_klass_volatility(data: &[MarketData]) -> f64 {
    if data.len() < 2 {
        println!("Insufficient data: {} intervals", data.len());
        return 0.0;
    }

    let mut volatilities = Vec::new();
    let mut total_time = 0.0;
    let mut valid_intervals = 0;
    let mut hl_ranges = Vec::new();

    for i in 0..data.len() - 1 {
        let current = data[i];
        let next = data[i + 1];

        let duration = (next.timestamp - current.timestamp) / 3_600_000_000_000.0; // µs to hours
        if duration <= 0.0 || duration > 2.0 {
            println!("Invalid duration at index {}: {} hours (ts: {} to {})", i, duration, current.timestamp, next.timestamp);
            continue;
        }

        if current.high > current.low && current.open > 0.0 && current.close > 0.0
            && current.high.is_finite() && current.low.is_finite()
            && current.open.is_finite() && current.close.is_finite()
        {
            let hl_ratio = (current.high - current.low) / current.low * 100.0;
            let log_hl = (current.high / current.low).ln();
            let log_co = (current.close / current.open).ln();
            let volatility_term = 0.5 * log_hl * log_hl - (2.0 * 2.0_f64.ln() - 1.0) * log_co * log_co;
            if volatility_term >= 0.0 {
                let volatility = volatility_term.sqrt();
                if volatility.is_finite() {
                    volatilities.push((volatility, duration));
                    total_time += duration;
                    valid_intervals += 1;
                    hl_ranges.push(hl_ratio);
                }
            }
        }
    }

    if volatilities.is_empty() {
        println!("No valid intervals after filtering high=low");
        return 0.0;
    }

    let weighted_vol = volatilities
        .iter()
        .map(|&(vol, duration)| vol * duration)
        .sum::<f64>()
        / total_time;

    let hours_per_year = 8_760.0;
    let annualization_factor = (hours_per_year / total_time).sqrt();

    let avg_hl = if !hl_ranges.is_empty() {
        hl_ranges.iter().sum::<f64>() / hl_ranges.len() as f64
    } else {
        0.0
    };

    println!("Raw Garman-Klass Vol: {:.8}", weighted_vol);
    println!("Valid Intervals: {} ({:.2}%)", valid_intervals, valid_intervals as f64 / data.len() as f64 * 100.0);
    println!("Total Time (hours): {}", total_time);
    println!("Annualization Factor: {:.4}", annualization_factor);
    println!("Average High-Low Range: {:.6}%", avg_hl);

    weighted_vol * annualization_factor
}
#[tokio::main]
async fn main() {

    let data = vec![
        MarketData { timestamp: 1744243200000000.0, open: 82615.22, high: 82753.21, low: 82056.19, close: 82352.85 },
        MarketData { timestamp: 1744246800000000.0, open: 82352.85, high: 82600.0, low: 82224.9, close: 82249.29 },
        MarketData { timestamp: 1744250400000000.0, open: 82249.29, high: 82565.19, low: 82195.77, close: 82200.0 },
        MarketData { timestamp: 1744254000000000.0, open: 82201.24, high: 82224.45, low: 81472.84, close: 81775.2 },
        MarketData { timestamp: 1744257600000000.0, open: 81775.2, high: 82088.75, low: 81640.74, close: 82019.26 },
        MarketData { timestamp: 1744261200000000.0, open: 82019.26, high: 82241.4, low: 81924.96, close: 82158.03 },
        MarketData { timestamp: 1744264800000000.0, open: 82158.03, high: 82344.3, low: 81838.03, close: 82156.72 },
        MarketData { timestamp: 1744268400000000.0, open: 82156.72, high: 82250.0, low: 81292.99, close: 81595.81 },
        MarketData { timestamp: 1744272000000000.0, open: 81595.81, high: 81853.2, low: 81420.29, close: 81549.99 },
        MarketData { timestamp: 1744275600000000.0, open: 81550.0, high: 82079.57, low: 81349.0, close: 82036.71 },
        MarketData { timestamp: 1744279200000000.0, open: 82036.72, high: 82130.43, low: 81565.43, close: 81766.05 },
        MarketData { timestamp: 1744282800000000.0, open: 81766.04, high: 81944.0, low: 81502.02, close: 81882.01 },
        MarketData { timestamp: 1744286400000000.0, open: 81882.0, high: 82466.35, low: 81459.48, close: 81710.46 },
        MarketData { timestamp: 1744290000000000.0, open: 81710.45, high: 81948.0, low: 80764.84, close: 80874.36 },
        MarketData { timestamp: 1744293600000000.0, open: 80874.35, high: 81841.58, low: 80634.26, close: 81272.19 },
        MarketData { timestamp: 1744297200000000.0, open: 81272.19, high: 81353.1, low: 78606.06, close: 78757.0 },
        MarketData { timestamp: 1744300800000000.0, open: 78757.24, high: 79688.0, low: 78464.36, close: 79453.82 },
        MarketData { timestamp: 1744304400000000.0, open: 79453.83, high: 79672.46, low: 78616.0, close: 79328.01 },
        MarketData { timestamp: 1744308000000000.0, open: 79328.01, high: 79876.61, low: 79223.62, close: 79502.07 },
        MarketData { timestamp: 1744311600000000.0, open: 79502.08, high: 80058.96, low: 79304.59, close: 79649.34 },
        MarketData { timestamp: 1744315200000000.0, open: 79647.37, high: 79960.97, low: 79570.12, close: 79931.55 },
        MarketData { timestamp: 1744318800000000.0, open: 79931.56, high: 80035.54, low: 79683.81, close: 79714.52 },
        MarketData { timestamp: 1744322400000000.0, open: 79714.51, high: 79920.0, low: 79545.45, close: 79693.77 },
        MarketData { timestamp: 1744326000000000.0, open: 79693.77, high: 79743.33, low: 79377.04, close: 79607.3 },
        MarketData { timestamp: 1744329600000000.0, open: 79607.3, high: 79750.17, low: 78969.58, close: 79043.48 },
        MarketData { timestamp: 1744333200000000.0, open: 79043.48, high: 80459.01, low: 79011.18, close: 80452.72 },
        MarketData { timestamp: 1744336800000000.0, open: 80452.71, high: 80829.02, low: 80162.18, close: 80302.05 },
        MarketData { timestamp: 1744340400000000.0, open: 80302.05, high: 80906.63, low: 80295.72, close: 80842.99 },
        MarketData { timestamp: 1744344000000000.0, open: 80843.0, high: 81100.0, low: 80720.07, close: 80888.15 },
        MarketData { timestamp: 1744347600000000.0, open: 80888.14, high: 80912.02, low: 80512.22, close: 80816.0 },
        MarketData { timestamp: 1744351200000000.0, open: 80815.99, high: 81333.0, low: 80815.99, close: 81102.0 },
        MarketData { timestamp: 1744354800000000.0, open: 81102.0, high: 81440.73, low: 80850.0, close: 81266.76 },
        MarketData { timestamp: 1744358400000000.0, open: 81266.76, high: 81936.71, low: 80782.43, close: 81631.89 },
        MarketData { timestamp: 1744362000000000.0, open: 81631.89, high: 82450.23, low: 81450.0, close: 82450.0 },
        MarketData { timestamp: 1744365600000000.0, open: 82450.0, high: 82985.71, low: 82304.34, close: 82706.86 },
        MarketData { timestamp: 1744369200000000.0, open: 82706.85, high: 82901.02, low: 82020.25, close: 82129.67 },
        MarketData { timestamp: 1744372800000000.0, open: 82129.67, high: 82564.0, low: 81819.14, close: 82115.52 },
        MarketData { timestamp: 1744376400000000.0, open: 82115.53, high: 83333.0, low: 81816.01, close: 82930.67 },
        MarketData { timestamp: 1744380000000000.0, open: 82930.67, high: 82930.67, low: 81341.22, close: 81898.0 },
        MarketData { timestamp: 1744383600000000.0, open: 81898.0, high: 82700.0, low: 81716.0, close: 82521.74 },
        MarketData { timestamp: 1744387200000000.0, open: 82521.74, high: 83022.98, low: 82015.08, close: 82792.46 },
        MarketData { timestamp: 1744390800000000.0, open: 82792.46, high: 84073.0, low: 82695.65, close: 83930.0 },
        MarketData { timestamp: 1744394400000000.0, open: 83930.0, high: 84220.0, low: 83520.0, close: 83607.99 },
        MarketData { timestamp: 1744398000000000.0, open: 83608.0, high: 83988.49, low: 83565.21, close: 83858.28 },
        MarketData { timestamp: 1744401600000000.0, open: 83858.28, high: 84029.13, low: 83716.98, close: 83848.0 },
        MarketData { timestamp: 1744405200000000.0, open: 83848.01, high: 84300.0, low: 83636.36, close: 83674.01 },
        MarketData { timestamp: 1744408800000000.0, open: 83674.01, high: 83716.99, low: 83316.36, close: 83415.34 },
        MarketData { timestamp: 1744412400000000.0, open: 83415.35, high: 83597.94, low: 83162.0, close: 83423.84 },
        MarketData { timestamp: 1744416000000000.0, open: 83423.83, high: 83572.0, low: 83245.89, close: 83369.08 },
        MarketData { timestamp: 1744419600000000.0, open: 83369.08, high: 83453.22, low: 83144.57, close: 83217.39 },
        MarketData { timestamp: 1744423200000000.0, open: 83217.39, high: 83310.05, low: 82800.01, close: 82804.87 },
        MarketData { timestamp: 1744426800000000.0, open: 82804.86, high: 83140.0, low: 82792.95, close: 82973.13 },
        MarketData { timestamp: 1744430400000000.0, open: 82973.14, high: 83464.0, low: 82856.86, close: 83208.04 },
        MarketData { timestamp: 1744434000000000.0, open: 83208.04, high: 83506.39, low: 83198.11, close: 83338.92 },
        MarketData { timestamp: 1744437600000000.0, open: 83338.92, high: 83790.05, low: 83338.91, close: 83567.42 },
        MarketData { timestamp: 1744441200000000.0, open: 83567.43, high: 83830.62, low: 83496.56, close: 83724.92 },
        MarketData { timestamp: 1744444800000000.0, open: 83724.92, high: 84031.87, low: 83480.01, close: 83604.77 },
        MarketData { timestamp: 1744448400000000.0, open: 83604.77, high: 83766.74, low: 83420.0, close: 83550.0 },
        MarketData { timestamp: 1744452000000000.0, open: 83550.0, high: 83565.22, low: 83305.91, close: 83420.44 },
        MarketData { timestamp: 1744455600000000.0, open: 83420.44, high: 83647.93, low: 83420.44, close: 83528.3 },
        MarketData { timestamp: 1744459200000000.0, open: 83528.3, high: 83936.45, low: 83474.94, close: 83829.86 },
        MarketData { timestamp: 1744462800000000.0, open: 83829.86, high: 84688.0, low: 83555.12, close: 84458.24 },
        MarketData { timestamp: 1744466400000000.0, open: 84458.24, high: 85251.0, low: 84314.62, close: 84827.75 },
        MarketData { timestamp: 1744470000000000.0, open: 84827.75, high: 85402.9, low: 84624.73, close: 85008.4 },
        MarketData { timestamp: 1744473600000000.0, open: 85008.4, high: 85212.81, low: 84575.15, close: 84659.19 },
        MarketData { timestamp: 1744477200000000.0, open: 84659.18, high: 84867.51, low: 84421.95, close: 84760.71 },
        MarketData { timestamp: 1744480800000000.0, open: 84760.71, high: 85193.09, low: 84755.5, close: 85178.01 },
        MarketData { timestamp: 1744484400000000.0, open: 85178.0, high: 85258.29, low: 84834.75, close: 84943.4 },
        MarketData { timestamp: 1744488000000000.0, open: 84943.39, high: 85905.0, low: 84877.61, close: 85517.29 },
        MarketData { timestamp: 1744491600000000.0, open: 85517.28, high: 85566.28, low: 85260.86, close: 85367.93 },
        MarketData { timestamp: 1744495200000000.0, open: 85367.93, high: 85701.6, low: 85329.27, close: 85336.01 },
        MarketData { timestamp: 1744498800000000.0, open: 85336.01, high: 85428.97, low: 85170.0, close: 85276.9 },
        MarketData { timestamp: 1744502400000000.0, open: 85276.91, high: 85571.12, low: 85012.0, close: 85495.72 },
        MarketData { timestamp: 1744506000000000.0, open: 85495.72, high: 86100.0, low: 85142.46, close: 85259.77 },
        MarketData { timestamp: 1744509600000000.0, open: 85259.76, high: 85451.85, low: 84724.91, close: 85302.22 },
        MarketData { timestamp: 1744513200000000.0, open: 85302.21, high: 85724.79, low: 85247.03, close: 85432.31 },
        MarketData { timestamp: 1744516800000000.0, open: 85432.31, high: 85436.68, low: 84464.44, close: 84619.98 },
        MarketData { timestamp: 1744520400000000.0, open: 84619.98, high: 84804.57, low: 84399.42, close: 84515.58 },
        MarketData { timestamp: 1744524000000000.0, open: 84515.57, high: 84849.06, low: 84444.72, close: 84608.69 },
        MarketData { timestamp: 1744527600000000.0, open: 84608.7, high: 84769.87, low: 84413.09, close: 84413.1 },
        MarketData { timestamp: 1744531200000000.0, open: 84413.1, high: 84693.2, low: 84277.03, close: 84664.6 },
        MarketData { timestamp: 1744534800000000.0, open: 84664.61, high: 84948.25, low: 84504.58, close: 84918.24 },
        MarketData { timestamp: 1744538400000000.0, open: 84918.25, high: 84949.86, low: 84510.0, close: 84686.79 },
        MarketData { timestamp: 1744542000000000.0, open: 84686.8, high: 84771.25, low: 84216.15, close: 84528.66 },
        MarketData { timestamp: 1744545600000000.0, open: 84528.66, high: 84700.0, low: 84245.39, close: 84458.75 },
        MarketData { timestamp: 1744549200000000.0, open: 84458.75, high: 84620.01, low: 83487.72, close: 83599.13 },
        MarketData { timestamp: 1744552800000000.0, open: 83599.12, high: 84166.96, low: 83480.67, close: 84093.22 },
        MarketData { timestamp: 1744556400000000.0, open: 84093.22, high: 84093.23, low: 83665.3, close: 83849.66 },
        MarketData { timestamp: 1744560000000000.0, open: 83849.66, high: 84424.53, low: 83845.13, close: 84225.97 },
        MarketData { timestamp: 1744563600000000.0, open: 84225.97, high: 84990.22, low: 84190.04, close: 84754.85 },
        MarketData { timestamp: 1744567200000000.0, open: 84754.84, high: 84943.4, low: 84524.14, close: 84730.0 },
        MarketData { timestamp: 1744570800000000.0, open: 84730.0, high: 84730.0, low: 83050.0, close: 83981.31 },
        MarketData { timestamp: 1744574400000000.0, open: 83981.32, high: 84446.31, low: 83154.82, close: 83499.99 },
        MarketData { timestamp: 1744578000000000.0, open: 83499.98, high: 83826.09, low: 83034.23, close: 83607.0 },
        MarketData { timestamp: 1744581600000000.0, open: 83607.0, high: 83882.77, low: 83169.01, close: 83335.99 },
        MarketData { timestamp: 1744585200000000.0, open: 83335.99, high: 83840.43, low: 83169.03, close: 83760.0 },
    ];

    // Debug: Print first 5 Klines
    let sample = &data[0..data.len().min(5)];
    for (i, d) in sample.iter().enumerate() {
        println!(
            "Kline {}: ts: {}, O: {:.2}, H: {:.2}, L: {:.2}, C: {:.2}",
            i, d.timestamp, d.open, d.high, d.low, d.close
        );
    }

    let parkinson_vol = time_weighted_parkinson_volatility(&data);
    let garman_klass_vol = time_weighted_garman_klass_volatility(&data);

    println!("Annualized Parkinson Volatility: {:.2}%", parkinson_vol * 100.0);
    println!("Annualized Garman-Klass Volatility: {:.2}%", garman_klass_vol * 100.0);
    println!("Implied Volatility Mark: 53%");
    println!("Parkinson Difference: {:.2}%", parkinson_vol * 100.0 - 53.0);
    println!("Garman-Klass Difference: {:.2}%", garman_klass_vol * 100.0 - 53.0);

    let min_close = data.iter().map(|d| d.close).fold(f64::INFINITY, f64::min);
    let max_close = data.iter().map(|d| d.close).fold(f64::NEG_INFINITY, f64::max);
    let min_low = data.iter().map(|d| d.low).fold(f64::INFINITY, f64::min);
    let max_high = data.iter().map(|d| d.high).fold(f64::NEG_INFINITY, f64::max);
    let hl_ranges = data.iter().map(|d| (d.high - d.low) / d.low * 100.0).collect::<Vec<_>>();
    let avg_hl = hl_ranges.iter().sum::<f64>() / hl_ranges.len() as f64;
    let flat_hl = data.iter().filter(|d| d.high == d.low).count();
    let max_hl = hl_ranges.iter().fold(f64::NEG_INFINITY, |arg0: f64, other: &f64| f64::max(arg0, *other));
    let large_hl = data.iter().filter(|d| (d.high - d.low) / d.low * 100.0 >= 0.1).count();
    let tiny_hl = data.iter().filter(|d| (d.high - d.low) / d.low * 100.0 < 0.01 && d.high > d.low).count();
    let no_trades = data.iter().filter(|d| d.high == d.low && d.open == d.close && d.high == d.close).count();
    let mid_hl = data.iter().filter(|d| {
        let hl = (d.high - d.low) / d.low * 100.0;
        hl >= 0.01 && hl < 0.1 && d.high > d.low
    }).count();

    println!(
        "Close Range: ${:.2}–${:.2} ({:.2}%)",
        min_close,
        max_close,
        (max_close - min_close) / min_close * 100.0
    );
    println!(
        "High-Low Range: ${:.2}–${:.2} ({:.2}%)",
        min_low,
        max_high,
        (max_high - min_low) / min_low * 100.0
    );
    println!("Average 1-hour high-low range: {:.6}%", avg_hl);
    println!("Max 1-hour high-low range: {:.6}%", max_hl);
    println!("High=Low intervals: {} ({:.2}%)", flat_hl, flat_hl as f64 / data.len() as f64 * 100.0);
    println!("High-Low >=0.1% intervals: {} ({:.2}%)", large_hl, large_hl as f64 / data.len() as f64 * 100.0);
    println!("High-Low <0.01% intervals: {} ({:.2}%)", tiny_hl, tiny_hl as f64 / data.len() as f64 * 100.0);
    println!("High-Low 0.01–0.1% intervals: {} ({:.2}%)", mid_hl, mid_hl as f64 / data.len() as f64 * 100.0);
    println!("No-trade intervals: {} ({:.2}%)", no_trades, no_trades as f64 / data.len() as f64 * 100.0);
}

async fn get_data() -> Vec<MarketData> {
    // Your 4-day data (partial, replace with full ~345,600 entries)
    let data = btc::fetch_prev_ndays_1s_data(4).await.unwrap().iter()
        .map(|d| MarketData { timestamp: d.timestamp, close: d.close, low: d.low.unwrap(), high: d.high.unwrap(), open: d.open.unwrap() })
        .collect::<Vec<_>>();

    data
}*/


fn compute_volatility(data: &[MarketData]) {
    if data.len() < 2 {
        println!("Insufficient data: {} intervals", data.len());
        return;
    }

    let mut parkinson_vols = Vec::new();
    let mut garman_klass_vols = Vec::new();
    let mut total_time = 0.0;
    let mut valid_intervals = 0;
    let mut hl_ranges = Vec::new();

    for i in 0..data.len() - 1 {
        let current = data[i];
        let next = data[i + 1];

        // Debug divisor
        let duration = (next.timestamp * 1000.0  - current.timestamp * 1000.0) / 3_600_000f64; // Correct
        if i == 0 {
            println!("First duration: {} hours (ts diff: {})", duration, next.timestamp * 1000.0 - current.timestamp * 1000.0);
        }
        if duration <= 0.0 || duration > 2.0 {
            println!("Invalid duration at index {}: {} hours", i, duration);
            continue;
        }

        if current.high > current.low && current.high.is_finite() && current.low.is_finite() {
            let hl_ratio = (current.high - current.low) / current.low * 100.0;
            let log_hl = (current.high / current.low).ln();

            // Parkinson
            let parkinson = (log_hl * log_hl / (4.0 * 2.0_f64.ln())).sqrt();
            if parkinson.is_finite() {
                parkinson_vols.push((parkinson, duration));
            }

            // Garman-Klass
            if current.open > 0.0 && current.close > 0.0 && current.open.is_finite() && current.close.is_finite() {
                let log_co = (current.close / current.open).ln();
                let gk_term = 0.5 * log_hl * log_hl - (2.0 * 2.0_f64.ln() - 1.0) * log_co * log_co;
                if gk_term >= 0.0 {
                    let garman_klass = gk_term.sqrt();
                    if garman_klass.is_finite() {
                        garman_klass_vols.push((garman_klass, duration));
                    }
                }
            }

            total_time += duration;
            valid_intervals += 1;
            hl_ranges.push(hl_ratio);
        }
    }

    if parkinson_vols.is_empty() {
        println!("No valid intervals");
        return;
    }

    let parkinson_vol = parkinson_vols.iter().map(|&(vol, dur)| vol * dur).sum::<f64>() / total_time;
    let garman_klass_vol = garman_klass_vols.iter().map(|&(vol, dur)| vol * dur).sum::<f64>() / total_time;
    let annualization_factor = (8_760.0 / total_time).sqrt();
    let avg_hl = hl_ranges.iter().sum::<f64>() / hl_ranges.len() as f64;

    println!("Raw Parkinson Vol: {:.8}", parkinson_vol);
    println!("Raw Garman-Klass Vol: {:.8}", garman_klass_vol);
    println!("Valid Intervals: {} ({:.2}%)", valid_intervals, valid_intervals as f64 / data.len() as f64 * 100.0);
    println!("Total Time (hours): {}", total_time);
    println!("Annualization Factor: {:.4}", annualization_factor);
    println!("Average High-Low Range: {:.6}%", avg_hl);
    println!("Annualized Parkinson Volatility: {:.2}%", parkinson_vol * annualization_factor * 100.0);
    println!("Annualized Garman-Klass Volatility: {:.2}%", garman_klass_vol * annualization_factor * 100.0);
    println!("Implied Volatility Mark: 53%");
    println!("Parkinson Difference: {:.2}%", parkinson_vol * annualization_factor * 100.0 - 53.0);
    println!("Garman-Klass Difference: {:.2}%", garman_klass_vol * annualization_factor * 100.0 - 53.0);

    let min_close = data.iter().map(|d| d.close).fold(f64::INFINITY, f64::min);
    let max_close = data.iter().map(|d| d.close).fold(f64::NEG_INFINITY, f64::max);
    let min_low = data.iter().map(|d| d.low).fold(f64::INFINITY, f64::min);
    let max_high = data.iter().map(|d| d.high).fold(f64::NEG_INFINITY, f64::max);
    let hl_ranges = data.iter().map(|d| (d.high - d.low) / d.low * 100.0).collect::<Vec<_>>();
    let avg_hl = hl_ranges.iter().sum::<f64>() / hl_ranges.len() as f64;
    let flat_hl = data.iter().filter(|d| d.high == d.low).count();
    let max_hl = hl_ranges.iter().fold(f64::NEG_INFINITY, |arg0: f64, other: &f64| f64::max(arg0, *other));
    let large_hl = data.iter().filter(|d| (d.high - d.low) / d.low * 100.0 >= 0.1).count();
    let tiny_hl = data.iter().filter(|d| (d.high - d.low) / d.low * 100.0 < 0.01 && d.high > d.low).count();
    let no_trades = data.iter().filter(|d| d.high == d.low && d.open == d.close && d.high == d.close).count();
    let mid_hl = data.iter().filter(|d| {
        let hl = (d.high - d.low) / d.low * 100.0;
        hl >= 0.01 && hl < 0.1 && d.high > d.low
    }).count();

    println!(
        "Close Range: ${:.2}–${:.2} ({:.2}%)",
        min_close,
        max_close,
        (max_close - min_close) / min_close * 100.0
    );
    println!(
        "High-Low Range: ${:.2}–${:.2} ({:.2}%)",
        min_low,
        max_high,
        (max_high - min_low) / min_low * 100.0
    );
    println!("Average 1-hour high-low range: {:.6}%", avg_hl);
    println!("Max 1-hour high-low range: {:.6}%", max_hl);
    println!("High=Low intervals: {} ({:.2}%)", flat_hl, flat_hl as f64 / data.len() as f64 * 100.0);
    println!("High-Low >=0.1% intervals: {} ({:.2}%)", large_hl, large_hl as f64 / data.len() as f64 * 100.0);
    println!("High-Low <0.01% intervals: {} ({:.2}%)", tiny_hl, tiny_hl as f64 / data.len() as f64 * 100.0);
    println!("High-Low 0.01–0.1% intervals: {} ({:.2}%)", mid_hl, mid_hl as f64 / data.len() as f64 * 100.0);
    println!("No-trade intervals: {} ({:.2}%)", no_trades, no_trades as f64 / data.len() as f64 * 100.0);
}

fn main() {
    let data = vec![
        MarketData { timestamp: 1744243200000000.0, open: 82615.22, high: 82753.21, low: 82056.19, close: 82352.85 },
        MarketData { timestamp: 1744246800000000.0, open: 82352.85, high: 82600.0, low: 82224.9, close: 82249.29 },
        MarketData { timestamp: 1744250400000000.0, open: 82249.29, high: 82565.19, low: 82195.77, close: 82200.0 },
        MarketData { timestamp: 1744254000000000.0, open: 82201.24, high: 82224.45, low: 81472.84, close: 81775.2 },
        MarketData { timestamp: 1744257600000000.0, open: 81775.2, high: 82088.75, low: 81640.74, close: 82019.26 },
        MarketData { timestamp: 1744261200000000.0, open: 82019.26, high: 82241.4, low: 81924.96, close: 82158.03 },
        MarketData { timestamp: 1744264800000000.0, open: 82158.03, high: 82344.3, low: 81838.03, close: 82156.72 },
        MarketData { timestamp: 1744268400000000.0, open: 82156.72, high: 82250.0, low: 81292.99, close: 81595.81 },
        MarketData { timestamp: 1744272000000000.0, open: 81595.81, high: 81853.2, low: 81420.29, close: 81549.99 },
        MarketData { timestamp: 1744275600000000.0, open: 81550.0, high: 82079.57, low: 81349.0, close: 82036.71 },
        MarketData { timestamp: 1744279200000000.0, open: 82036.72, high: 82130.43, low: 81565.43, close: 81766.05 },
        MarketData { timestamp: 1744282800000000.0, open: 81766.04, high: 81944.0, low: 81502.02, close: 81882.01 },
        MarketData { timestamp: 1744286400000000.0, open: 81882.0, high: 82466.35, low: 81459.48, close: 81710.46 },
        MarketData { timestamp: 1744290000000000.0, open: 81710.45, high: 81948.0, low: 80764.84, close: 80874.36 },
        MarketData { timestamp: 1744293600000000.0, open: 80874.35, high: 81841.58, low: 80634.26, close: 81272.19 },
        MarketData { timestamp: 1744297200000000.0, open: 81272.19, high: 81353.1, low: 78606.06, close: 78757.0 },
        MarketData { timestamp: 1744300800000000.0, open: 78757.24, high: 79688.0, low: 78464.36, close: 79453.82 },
        MarketData { timestamp: 1744304400000000.0, open: 79453.83, high: 79672.46, low: 78616.0, close: 79328.01 },
        MarketData { timestamp: 1744308000000000.0, open: 79328.01, high: 79876.61, low: 79223.62, close: 79502.07 },
        MarketData { timestamp: 1744311600000000.0, open: 79502.08, high: 80058.96, low: 79304.59, close: 79649.34 },
        MarketData { timestamp: 1744315200000000.0, open: 79647.37, high: 79960.97, low: 79570.12, close: 79931.55 },
        MarketData { timestamp: 1744318800000000.0, open: 79931.56, high: 80035.54, low: 79683.81, close: 79714.52 },
        MarketData { timestamp: 1744322400000000.0, open: 79714.51, high: 79920.0, low: 79545.45, close: 79693.77 },
        MarketData { timestamp: 1744326000000000.0, open: 79693.77, high: 79743.33, low: 79377.04, close: 79607.3 },
        MarketData { timestamp: 1744329600000000.0, open: 79607.3, high: 79750.17, low: 78969.58, close: 79043.48 },
        MarketData { timestamp: 1744333200000000.0, open: 79043.48, high: 80459.01, low: 79011.18, close: 80452.72 },
        MarketData { timestamp: 1744336800000000.0, open: 80452.71, high: 80829.02, low: 80162.18, close: 80302.05 },
        MarketData { timestamp: 1744340400000000.0, open: 80302.05, high: 80906.63, low: 80295.72, close: 80842.99 },
        MarketData { timestamp: 1744344000000000.0, open: 80843.0, high: 81100.0, low: 80720.07, close: 80888.15 },
        MarketData { timestamp: 1744347600000000.0, open: 80888.14, high: 80912.02, low: 80512.22, close: 80816.0 },
        MarketData { timestamp: 1744351200000000.0, open: 80815.99, high: 81333.0, low: 80815.99, close: 81102.0 },
        MarketData { timestamp: 1744354800000000.0, open: 81102.0, high: 81440.73, low: 80850.0, close: 81266.76 },
        MarketData { timestamp: 1744358400000000.0, open: 81266.76, high: 81936.71, low: 80782.43, close: 81631.89 },
        MarketData { timestamp: 1744362000000000.0, open: 81631.89, high: 82450.23, low: 81450.0, close: 82450.0 },
        MarketData { timestamp: 1744365600000000.0, open: 82450.0, high: 82985.71, low: 82304.34, close: 82706.86 },
        MarketData { timestamp: 1744369200000000.0, open: 82706.85, high: 82901.02, low: 82020.25, close: 82129.67 },
        MarketData { timestamp: 1744372800000000.0, open: 82129.67, high: 82564.0, low: 81819.14, close: 82115.52 },
        MarketData { timestamp: 1744376400000000.0, open: 82115.53, high: 83333.0, low: 81816.01, close: 82930.67 },
        MarketData { timestamp: 1744380000000000.0, open: 82930.67, high: 82930.67, low: 81341.22, close: 81898.0 },
        MarketData { timestamp: 1744383600000000.0, open: 81898.0, high: 82700.0, low: 81716.0, close: 82521.74 },
        MarketData { timestamp: 1744387200000000.0, open: 82521.74, high: 83022.98, low: 82015.08, close: 82792.46 },
        MarketData { timestamp: 1744390800000000.0, open: 82792.46, high: 84073.0, low: 82695.65, close: 83930.0 },
        MarketData { timestamp: 1744394400000000.0, open: 83930.0, high: 84220.0, low: 83520.0, close: 83607.99 },
        MarketData { timestamp: 1744398000000000.0, open: 83608.0, high: 83988.49, low: 83565.21, close: 83858.28 },
        MarketData { timestamp: 1744401600000000.0, open: 83858.28, high: 84029.13, low: 83716.98, close: 83848.0 },
        MarketData { timestamp: 1744405200000000.0, open: 83848.01, high: 84300.0, low: 83636.36, close: 83674.01 },
        MarketData { timestamp: 1744408800000000.0, open: 83674.01, high: 83716.99, low: 83316.36, close: 83415.34 },
        MarketData { timestamp: 1744412400000000.0, open: 83415.35, high: 83597.94, low: 83162.0, close: 83423.84 },
        MarketData { timestamp: 1744416000000000.0, open: 83423.83, high: 83572.0, low: 83245.89, close: 83369.08 },
        MarketData { timestamp: 1744419600000000.0, open: 83369.08, high: 83453.22, low: 83144.57, close: 83217.39 },
        MarketData { timestamp: 1744423200000000.0, open: 83217.39, high: 83310.05, low: 82800.01, close: 82804.87 },
        MarketData { timestamp: 1744426800000000.0, open: 82804.86, high: 83140.0, low: 82792.95, close: 82973.13 },
        MarketData { timestamp: 1744430400000000.0, open: 82973.14, high: 83464.0, low: 82856.86, close: 83208.04 },
        MarketData { timestamp: 1744434000000000.0, open: 83208.04, high: 83506.39, low: 83198.11, close: 83338.92 },
        MarketData { timestamp: 1744437600000000.0, open: 83338.92, high: 83790.05, low: 83338.91, close: 83567.42 },
        MarketData { timestamp: 1744441200000000.0, open: 83567.43, high: 83830.62, low: 83496.56, close: 83724.92 },
        MarketData { timestamp: 1744444800000000.0, open: 83724.92, high: 84031.87, low: 83480.01, close: 83604.77 },
        MarketData { timestamp: 1744448400000000.0, open: 83604.77, high: 83766.74, low: 83420.0, close: 83550.0 },
        MarketData { timestamp: 1744452000000000.0, open: 83550.0, high: 83565.22, low: 83305.91, close: 83420.44 },
        MarketData { timestamp: 1744455600000000.0, open: 83420.44, high: 83647.93, low: 83420.44, close: 83528.3 },
        MarketData { timestamp: 1744459200000000.0, open: 83528.3, high: 83936.45, low: 83474.94, close: 83829.86 },
        MarketData { timestamp: 1744462800000000.0, open: 83829.86, high: 84688.0, low: 83555.12, close: 84458.24 },
        MarketData { timestamp: 1744466400000000.0, open: 84458.24, high: 85251.0, low: 84314.62, close: 84827.75 },
        MarketData { timestamp: 1744470000000000.0, open: 84827.75, high: 85402.9, low: 84624.73, close: 85008.4 },
        MarketData { timestamp: 1744473600000000.0, open: 85008.4, high: 85212.81, low: 84575.15, close: 84659.19 },
        MarketData { timestamp: 1744477200000000.0, open: 84659.18, high: 84867.51, low: 84421.95, close: 84760.71 },
        MarketData { timestamp: 1744480800000000.0, open: 84760.71, high: 85193.09, low: 84755.5, close: 85178.01 },
        MarketData { timestamp: 1744484400000000.0, open: 85178.0, high: 85258.29, low: 84834.75, close: 84943.4 },
        MarketData { timestamp: 1744488000000000.0, open: 84943.39, high: 85905.0, low: 84877.61, close: 85517.29 },
        MarketData { timestamp: 1744491600000000.0, open: 85517.28, high: 85566.28, low: 85260.86, close: 85367.93 },
        MarketData { timestamp: 1744495200000000.0, open: 85367.93, high: 85701.6, low: 85329.27, close: 85336.01 },
        MarketData { timestamp: 1744498800000000.0, open: 85336.01, high: 85428.97, low: 85170.0, close: 85276.9 },
        MarketData { timestamp: 1744502400000000.0, open: 85276.91, high: 85571.12, low: 85012.0, close: 85495.72 },
        MarketData { timestamp: 1744506000000000.0, open: 85495.72, high: 86100.0, low: 85142.46, close: 85259.77 },
        MarketData { timestamp: 1744509600000000.0, open: 85259.76, high: 85451.85, low: 84724.91, close: 85302.22 },
        MarketData { timestamp: 1744513200000000.0, open: 85302.21, high: 85724.79, low: 85247.03, close: 85432.31 },
        MarketData { timestamp: 1744516800000000.0, open: 85432.31, high: 85436.68, low: 84464.44, close: 84619.98 },
        MarketData { timestamp: 1744520400000000.0, open: 84619.98, high: 84804.57, low: 84399.42, close: 84515.58 },
        MarketData { timestamp: 1744524000000000.0, open: 84515.57, high: 84849.06, low: 84444.72, close: 84608.69 },
        MarketData { timestamp: 1744527600000000.0, open: 84608.7, high: 84769.87, low: 84413.09, close: 84413.1 },
        MarketData { timestamp: 1744531200000000.0, open: 84413.1, high: 84693.2, low: 84277.03, close: 84664.6 },
        MarketData { timestamp: 1744534800000000.0, open: 84664.61, high: 84948.25, low: 84504.58, close: 84918.24 },
        MarketData { timestamp: 1744538400000000.0, open: 84918.25, high: 84949.86, low: 84510.0, close: 84686.79 },
        MarketData { timestamp: 1744542000000000.0, open: 84686.8, high: 84771.25, low: 84216.15, close: 84528.66 },
        MarketData { timestamp: 1744545600000000.0, open: 84528.66, high: 84700.0, low: 84245.39, close: 84458.75 },
        MarketData { timestamp: 1744549200000000.0, open: 84458.75, high: 84620.01, low: 83487.72, close: 83599.13 },
        MarketData { timestamp: 1744552800000000.0, open: 83599.12, high: 84166.96, low: 83480.67, close: 84093.22 },
        MarketData { timestamp: 1744556400000000.0, open: 84093.22, high: 84093.23, low: 83665.3, close: 83849.66 },
        MarketData { timestamp: 1744560000000000.0, open: 83849.66, high: 84424.53, low: 83845.13, close: 84225.97 },
        MarketData { timestamp: 1744563600000000.0, open: 84225.97, high: 84990.22, low: 84190.04, close: 84754.85 },
        MarketData { timestamp: 1744567200000000.0, open: 84754.84, high: 84943.4, low: 84524.14, close: 84730.0 },
        MarketData { timestamp: 1744570800000000.0, open: 84730.0, high: 84730.0, low: 83050.0, close: 83981.31 },
        MarketData { timestamp: 1744574400000000.0, open: 83981.32, high: 84446.31, low: 83154.82, close: 83499.99 },
        MarketData { timestamp: 1744578000000000.0, open: 83499.98, high: 83826.09, low: 83034.23, close: 83607.0 },
        MarketData { timestamp: 1744581600000000.0, open: 83607.0, high: 83882.77, low: 83169.01, close: 83335.99 },
        MarketData { timestamp: 1744585200000000.0, open: 83335.99, high: 83840.43, low: 83169.03, close: 83760.0 },
    ];

    let sample = &data[0..5];
    for (i, d) in sample.iter().enumerate() {
        println!(
            "Kline {}: ts: {}, O: {:.2}, H: {:.2}, L: {:.2}, C: {:.2}",
            i, d.timestamp, d.open, d.high, d.low, d.close
        );
    }

    compute_volatility(&data);
}