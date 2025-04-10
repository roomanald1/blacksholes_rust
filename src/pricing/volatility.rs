use std::collections::VecDeque;
use crate::data::btc_fetch::{MarketData};

pub struct TimeAwareEwmaVolatility {
    base_lambda: f64,
    base_period_secs: f64,
    hour_base_secs: f64,
    base_annualization_factor: f64,
    max_annualization_factor: f64,
    prev_vol: Option<f64>,
    prev_close: Option<f64>,
    prev_timestamp: Option<f64>,
    init_window: VecDeque<MarketData>,
    min_samples: usize,
}

impl TimeAwareEwmaVolatility {
    pub fn new(base_lambda: f64, min_samples: usize) -> Self {
        Self {
            base_lambda,
            base_period_secs: 86400.0,
            hour_base_secs: 3600.0,
            base_annualization_factor: 252.0,
            max_annualization_factor: 907_200.0,
            prev_vol: None,
            prev_close: None,
            prev_timestamp: None,
            init_window: VecDeque::with_capacity(min_samples),
            min_samples,
        }
    }

    fn scaled_variance_contribution(&self, price_now: f64, price_prev: f64, t_now: f64, t_prev: f64) -> (f64, f64) {
        if t_now <= t_prev || price_prev <= 0.0 || price_now <= 0.0 {
            //println!("Invalid inputs: t_now={:.1}, t_prev={:.1}, price_now={:.2}, price_prev={:.2}, returning 0.0", t_now, t_prev, price_now, price_prev);
            return (0.0, self.base_lambda);
        }
        let time_diff_secs = t_now - t_prev;
        let effective_lambda = self.base_lambda * (time_diff_secs / self.base_period_secs).min(1.0);
        let annualization_factor = (self.base_annualization_factor * self.hour_base_secs / time_diff_secs)
            .max(self.base_annualization_factor)
            .min(self.max_annualization_factor);
        let raw_return = (price_now / price_prev).ln();
        let variance_contrib = raw_return.powi(2) * annualization_factor;
        //println!("scaled_variance_contribution: price_now={:.2}, price_prev={:.2}, t_now={:.1}, t_prev={:.1}, time_diff_secs={:.1}, annualization_factor={:.1}, effective_lambda={:.6}, raw_return={:.6}, variance_contrib={:.6}", price_now, price_prev, t_now, t_prev, time_diff_secs, annualization_factor, effective_lambda, raw_return, variance_contrib);
        (variance_contrib, effective_lambda)
    }

    fn init_from_window(&mut self) {
        if self.init_window.len() < 2 {
            return;
        }
        //println!("Initializing from window: {:?}", self.init_window);
        let mut returns = Vec::with_capacity(self.init_window.len() - 1);
        let mut window_iter = self.init_window.iter().peekable();
        let mut prev = window_iter.next().unwrap();

        while let Some(curr) = window_iter.next() {
            if curr.timestamp > prev.timestamp {
                let raw_return = (curr.close / prev.close).ln();
                returns.push(raw_return);
            }
            prev = curr;
        }

        if returns.is_empty() {
            return;
        }

        let annualized_variance = if returns.len() == 1 {
            // Single return: use population variance (no mean adjustment)
            returns[0].powi(2) * self.base_annualization_factor
        } else {
            // Multiple returns: use sample variance
            let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
            let variance = returns.iter().map(|r| (r - mean_return).powi(2)).sum::<f64>() / (returns.len() - 1) as f64;
            variance * self.base_annualization_factor
        };
        let vol = annualized_variance.sqrt();
        self.prev_vol = Some(vol);
        self.prev_close = self.init_window.back().map(|d| d.close);
        self.prev_timestamp = self.init_window.back().map(|d| d.timestamp);
        self.init_window.clear();
        //println!("Initial vol: {:.6}%", vol * 100.0);
    }

    pub fn update(&mut self, data: &MarketData) -> Option<f64> {
        let timestamp = data.timestamp / 1e6;
        let close = data.close;
        //println!("update: timestamp={:.1} (raw={:.1}), close={:.2}", timestamp, data.timestamp, close);

        if self.prev_vol.is_none() {
            self.init_window.push_back(MarketData {
                timestamp,
                open: data.open,
                high: data.high,
                low: data.low,
                close,
            });
            if self.init_window.len() >= self.min_samples {
                self.init_from_window();
            }
            return self.prev_vol;
        }

        let prev_timestamp = self.prev_timestamp.unwrap_or(timestamp);
        if timestamp <= prev_timestamp {
            //println!("Duplicate or earlier timestamp detected: {:.1} <= {:.1}, skipping update", timestamp, prev_timestamp);
            return self.prev_vol;
        }

        let prev_close = self.prev_close.unwrap_or(close);
        let (variance_contrib, effective_lambda) = self.scaled_variance_contribution(close, prev_close, timestamp, prev_timestamp);
        let decay_factor = 1.0 - effective_lambda;

        let new_vol = match self.prev_vol {
            Some(prev) => {
                let new_variance = effective_lambda * variance_contrib + decay_factor * prev.powi(2);
                let vol = new_variance.sqrt();
                //println!("EWMA update: prev_vol={:.6}%, variance_contrib={:.6}, effective_lambda={:.6}, new_variance={:.6}, new_vol={:.6}%", prev * 100.0, variance_contrib, effective_lambda, new_variance, vol * 100.0);
                vol
            }
            None => variance_contrib.sqrt(),
        };

        self.prev_vol = Some(new_vol);
        self.prev_close = Some(close);
        self.prev_timestamp = Some(timestamp);
        Some(new_vol)
    }
}