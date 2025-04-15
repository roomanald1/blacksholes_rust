use serde::Deserialize;
use crate::stream::Price;

#[derive(Debug, Deserialize, Clone)]
pub struct MarketData {
    pub timestamp: f64,
    pub open: Option<f64>,
    pub high: Option<f64>,
    pub low: Option<f64>,
    pub close: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeribitDepthResponse {
    pub result: DeribitDepth
}

#[derive(Debug, Deserialize, Clone)]
pub struct DeribitSpotResponse {
    pub result: DeribitSpot
}
#[derive(Debug, Deserialize, Clone)]
pub struct DeribitSpot {
    pub index_price: f64
}


#[derive(Debug, Deserialize, Clone)]
pub struct DeribitDepth {
    pub index_price: f64,
    pub greeks: DeribitDepthsGreeks,
    pub mark_iv: f64,
    pub bids: Vec<Vec<f64>>,
    pub asks: Vec<Vec<f64>>,
}

#[derive(Debug)]
pub struct PriceAmount {
    pub price: f64,
    pub amount: f64
}

pub fn get_discrepancy(bid: PriceAmount, ask: PriceAmount, model_premium: f64) -> f64 {
    let market_premium = (bid.price + ask.price) / 2.0;
    let discrepancy = ((model_premium - market_premium) / market_premium) * 100.0;
    discrepancy
}

pub fn get_depth_info(response: DeribitDepthResponse) -> (PriceAmount, PriceAmount, f64){
    let best_bid = response.result.bids.first().unwrap().clone()[0]* response.result.index_price;
    let best_bid_volume = response.result.bids.first().unwrap().clone()[1];
    let best_ask = response.result.asks.first().unwrap().clone()[0]* response.result.index_price;
    let best_ask_volume = response.result.asks.first().unwrap().clone()[1];
    let spread= best_ask - best_bid;
    (
        PriceAmount{price: best_bid, amount: best_bid_volume},
        PriceAmount{price: best_ask, amount: best_ask_volume},
        spread,
    )
}

pub fn get_depth_data(response: DeribitDepthResponse) -> (Vec<PriceAmount>, Vec<PriceAmount>){
    (
        //Bids
        response.result.bids.iter().map(|b| PriceAmount{
            price: b.first().unwrap() * response.result.index_price,
            amount: b.last().unwrap().clone()
        }).collect(),
        //Asks
        response.result.asks.iter().map(|b| PriceAmount{
            price: b.first().unwrap() * response.result.index_price,
            amount: b.last().unwrap().clone()
        }).collect()
    )
}



#[derive(Debug, Deserialize, Clone)]
pub struct DeribitDepthsGreeks {
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
}

pub mod btc {
    use crate::data::btc_fetch::{DeribitDepthResponse, DeribitSpotResponse, MarketData};
    use zip::read::ZipArchive;
    use csv::ReaderBuilder;
    use std::io::{Cursor, Read};
    use chrono::{Duration, Utc};
    use futures_signals::signal::Signal;
    use tokio::time::sleep;
    use std::time::Duration as StdDuration;
    use futures::future::try_join_all;
    use tokio::task;
    use crate::utils::OptionType;


    pub async fn fetch_spot() -> Result<DeribitSpotResponse, String> {
        let url = format!("https://www.deribit.com/api/v2/public/get_index_price?index_name=btc_usd");
        let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
        let response : DeribitSpotResponse =  response.json::<DeribitSpotResponse>().await.map_err(|e| e.to_string())?;
        Ok(response)
    }

    pub async fn fetch_prev_ndays_1s_data(n: i64) -> Result<Vec<MarketData>, String> {
        let today = Utc::now().date_naive();
        let start_date = today - Duration::days(n);


        // Generate all URLs first (synchronous)
        let urls: Vec<String> = (0..n)
            .map(|i| {
                let date = start_date + Duration::days(i);
                format!(
                    "https://data.binance.vision/data/spot/daily/klines/BTCUSDT/1h/BTCUSDT-1h-{}.zip",
                    date.format("%Y-%m-%d")
                )
            })
            .collect();

        let tasks = urls.into_iter().map(|url| {
            task::spawn(async move {
                match fetch_and_parse(&url).await {
                    Ok(data) => Ok::<Vec<MarketData>, String>(data),
                    Err(e) => {
                        eprintln!("Failed to fetch {}: {}", url, e);
                        Ok(Vec::new()) // Skip errors, return empty Vec
                    }
                }
            })
        });

        // Await all tasks and flatten results
        let results: Vec<Vec<MarketData>> = try_join_all(tasks)
            .await
            .map_err(|e| format!("Join error: {}", e))?
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        let mut r: Vec<MarketData> = results.into_iter().flatten().collect();
        r.sort_by(|a,b| {
            a.timestamp.partial_cmp(&b.timestamp).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(r)
    }

    pub async fn fetch_prev_ndays(n: i64) -> Result<Vec<MarketData>, String> {
        let today = Utc::now().date_naive();
        let start_date = today - Duration::days(n);

        // Generate all URLs first (synchronous)
        let urls: Vec<String> = (0..n)
            .map(|i| {
                let date = start_date + Duration::days(i);
                format!(
                    "https://data.binance.vision/data/spot/daily/klines/BTCUSDT/1d/BTCUSDT-1d-{}.zip",
                    date.format("%Y-%m-%d")
                )
            })
            .collect();

        let tasks = urls.into_iter().map(|url| {
            task::spawn(async move {
                match fetch_and_parse(&url).await {
                    Ok(data) => Ok::<Vec<MarketData>, String>(data),
                    Err(e) => {
                        eprintln!("Failed to fetch {}: {}", url, e);
                        Ok(Vec::new()) // Skip errors, return empty Vec
                    }
                }
            })
        });

        // Await all tasks and flatten results
        let results: Vec<Vec<MarketData>> = try_join_all(tasks)
            .await
            .map_err(|e| format!("Join error: {}", e))?
            .into_iter()
            .filter_map(Result::ok)
            .collect();
        let mut r: Vec<MarketData> = results.into_iter().flatten().collect();
        r.sort_by(|a,b| {
            a.timestamp.partial_cmp(&b.timestamp).unwrap_or(std::cmp::Ordering::Equal)
        });
        Ok(r)
    }

    async fn fetch_and_parse(url: &str) -> Result<Vec<MarketData>, String> {
        let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
        let bytes = response.bytes().await.map_err(|e| e.to_string())?;
        let cursor = Cursor::new(bytes);

        // Extract and parse CSV
        let mut archive = ZipArchive::new(cursor).map_err(|e| e.to_string())?;
        let mut file = archive.by_index(0).map_err(|e| e.to_string())?;
        let mut csv_content = String::new();
        file.read_to_string(&mut csv_content).map_err(|e| e.to_string())?;

        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_content.as_bytes());

        let mut records = Vec::new();
        for result in rdr.deserialize() {
            records.push(result.map_err(|e| e.to_string())?);
        }
        Ok(records)
    }

    pub async fn fetch_prev_day_1s() -> Result<Vec<MarketData>, String> {
        let date = Utc::now().date_naive() - Duration::days(1);
        let url = format!(
            "https://data.binance.vision/data/spot/daily/klines/BTCUSDT/1s/BTCUSDT-1s-{}.zip",
            date.format("%Y-%m-%d")
        );
        fetch_and_parse(url.as_str()).await
    }


    //expiry = 27JUN25
    pub async fn fetch_live_option_price(strike: f64, expiry: String, optionType: OptionType) -> Result<DeribitDepthResponse, String> {
        let url = format!("https://www.deribit.com/api/v2/public/get_order_book?instrument_name=BTC-{}-{}-{}",
                          expiry.to_uppercase().as_str(),
                          strike,
                          if optionType == OptionType::Call {"C"}else {"P"});
        let response = reqwest::get(url).await.map_err(|e| e.to_string())?;
        let response : DeribitDepthResponse =  response.json::<DeribitDepthResponse>().await.map_err(|e| e.to_string())?;
        Ok(response)
    }

    pub async fn create_1s_ticking(date_offset: i64) -> impl Signal<Item = Option<MarketData>> {
        let mut index = 0;

        let date = Utc::now().date_naive() - Duration::days(date_offset);
        let url = format!(
            "https://data.binance.vision/data/spot/daily/klines/BTCUSDT/1s/BTCUSDT-1s-{}.zip",
            date.format("%Y-%m-%d")
        );
        let task = task::spawn(async move {
                match fetch_and_parse(&url).await {
                    Ok(data) => Ok::<Vec<MarketData>, String>(data.into_iter().skip(1).collect()),
                    Err(e) => {
                        eprintln!("Failed to fetch {}: {}", url, e);
                        Ok(Vec::new()) // Skip errors, return empty Vec
                    }
                }
            });

        // Await all tasks and flatten results
        let results: Vec<Vec<MarketData>> = try_join_all(vec![task])
            .await
            .map_err(|e| format!("Join error: {}", e))
            .unwrap()
            .into_iter()
            .filter_map(Result::ok)
            .collect();

        let market_data: Vec<MarketData> = results.into_iter().flatten().collect();

        futures_signals::signal::from_stream(async_stream::stream! {
            loop {
               sleep(StdDuration::from_secs_f64(1.0)).await;
               if let Some(md) = market_data.get(index){
                    yield md.clone();
               }
               index += 1;
            }
        })
    }
}



#[cfg(test)]
mod tests {
    use crate::data::btc_fetch::btc;

    #[test]
    fn gets_n_days_data() {
        let data = btc::fetch_prev_ndays(30).unwrap();
        assert_eq!(data.len(), 30);
    }
}
