use crate::data::btc_fetch::{btc, get_depth_data, PriceAmount};
use crate::utils::OptionType;


#[derive(Debug)]
pub struct OptionSide {
    pub strike: f64,
    pub expiry: String,
    pub option_type: OptionType,
    pub mark_iv: Option<f64>,
    pub bids: Option<Vec<PriceAmount>>,
    pub asks: Option<Vec<PriceAmount>>,
    pub latest_index_price: Option<f64>
}

pub struct OptionContract {
    pub expiry: String,
    pub strike: f64,
    pub latest_index_price: Option<f64>,
    pub call: OptionSide,
    pub put: OptionSide
}

impl OptionSide {
    pub fn new(strike: f64, expiry: String, option_type: OptionType) -> OptionSide {
        OptionSide {
            strike,
            expiry,
            option_type,
            mark_iv: None,
            bids: None,
            asks: None,
            latest_index_price: None
        }
    }

    pub async fn eval(&mut self) -> Result<(), String>{
        let response = btc::fetch_live_option_price(
                self.strike,
                self.expiry.clone(),
                self.option_type)
            .await.map_err(|e| e.to_string())?;

        self.mark_iv = Some(response.result.mark_iv);
        self.latest_index_price = Some(response.result.mark_iv);
        let (bids, asks) = get_depth_data(response.clone());
        self.bids = Some(bids);
        self.asks = Some(asks);

        Ok(())
    }
}




impl OptionContract {
    pub fn new(expiry: String, strike: f64) -> OptionContract {
        OptionContract{
            expiry: expiry.clone(),
            strike,
            latest_index_price: None,
            call: OptionSide::new(strike, expiry.clone(), OptionType::Call),
            put: OptionSide::new(strike, expiry, OptionType::Put)
        }
    }

    pub async fn eval(&mut self) -> Result<(), String>{

        self.call.eval().await.map_err(|e| e.to_string())?;
        self.put.eval().await.map_err(|e| e.to_string())?;

        self.latest_index_price = self.put.latest_index_price.or_else(|| self.call.latest_index_price);
        Ok(())
    }

}
