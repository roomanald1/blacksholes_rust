use std::cmp::Ordering;
use std::io;
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use chrono::NaiveDate;
use dhat::Profiler;
use blacksholes_rust::data::btc_fetch::btc::fetch_live_option_price;
use blacksholes_rust::pricing::option_pricer::OptionPricer;
use blacksholes_rust::{stream, utils};
use blacksholes_rust::data::btc_fetch::{get_asks, get_bids};
use blacksholes_rust::stream::start_streaming;
use blacksholes_rust::utils::OptionType;

#[tokio::main]
async fn main()  {


    let expiry = NaiveDate::from_ymd(2025, 6, 27);
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