use statrs::statistics::Statistics;
use blacksholes_rust::data::btc_fetch::btc;
use blacksholes_rust::pricing::volatility::Garch11;

#[derive(Clone, Copy, Debug)]
struct MarketData {
    timestamp: f64, // In microseconds
    open: f64,
    high: f64,
    low: f64,
    close: f64,
}

fn compute_volatility(data: &[MarketData]) -> f64 {
    if data.len() < 2 {
        println!("Insufficient data: {} intervals", data.len());
        return 0.0;
    }

    println!("Using divisor: 3_600_000_000.0 (µs to hours)");

    let mut parkinson_vols = Vec::new();
    let mut garman_klass_vols = Vec::new();
    let mut total_time = 0.0;
    let mut valid_intervals = 0;

    for i in 0..data.len() - 1 {
        let current = data[i];
        let next = data[i + 1];

        let duration = (next.timestamp - current.timestamp) / 3_600_000_000.0; // Fixed conversion
        if i == 0 {
            println!("First duration: {} hours (ts diff: {}) {} {}", duration, next.timestamp - current.timestamp, next.timestamp, current.timestamp);
        }

        if duration <= 0.0 || duration > 2.0 {
            println!("Invalid duration at index {}: {} hours", i, duration);
            continue;
        }

        if current.high > current.low && current.high.is_finite() && current.low.is_finite() {
            let log_hl = (current.high / current.low).ln();

            // Parkinson volatility for this interval
            let parkinson = (log_hl * log_hl / (4.0 * 2.0_f64.ln())).sqrt();
            if parkinson.is_finite() {
                parkinson_vols.push((parkinson, duration));
            }

            // Garman-Klass volatility for this interval
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
        }
    }

    if parkinson_vols.is_empty() {
        println!("No valid intervals");
        return 0.0;
    }

    // Calculate weighted average volatility
    let parkinson_vol = parkinson_vols.iter().map(|&(vol, dur)| vol * vol * dur).sum::<f64>().sqrt() / total_time.sqrt();
    let garman_klass_vol = garman_klass_vols.iter().map(|&(vol, dur)| vol * vol * dur).sum::<f64>().sqrt() / total_time.sqrt();

    // Annualization factor (using 2016 trading hours for 24/7 crypto)
    let annualization_factor = (8760.0f64).sqrt();

    println!("Raw Parkinson Vol: {:.8}", parkinson_vol);
    println!("Raw Garman-Klass Vol: {:.8}", garman_klass_vol);
    println!("Valid Intervals: {}", valid_intervals);
    println!("Total Time (hours): {}", total_time);
    println!("Annualization Factor: {:.4}", annualization_factor);
    println!("Annualized Parkinson Volatility: {:.2}%", parkinson_vol * annualization_factor * 100.0);
    println!("Annualized Garman-Klass Volatility: {:.2}%", garman_klass_vol * annualization_factor * 100.0);
    //garman_klass_vol * annualization_factor * 100.0
    parkinson_vol * annualization_factor * 100.0
}

async fn get_data() -> Vec<MarketData> {
    // Your 4-day data (partial, replace with full ~345,600 entries)
    let data = btc::fetch_prev_ndays_1s_data(10).await.unwrap().iter()
        .map(|d| MarketData { timestamp: d.timestamp, close: d.close, low: d.low.unwrap(), high: d.high.unwrap(), open: d.open.unwrap() })
        .collect::<Vec<_>>();

    data
}



#[derive(Clone, Copy, Debug)]
struct GarchParams {
    omega: f64,
    alpha: f64,
    beta: f64,
}

struct GarchModel {
    // GARCH(1,1) parameters
    omega: f64,          // Long-term average variance
    alpha: f64,          // Reaction to shocks (0.05-0.15 typical)
    beta: f64,           // Persistence (0.8-0.9 typical)
    // State variables
    last_variance: f64,  // Current variance estimate
    last_price: f64,     // Last observed price
    long_run_variance: f64, // Theoretical long-run variance
}

impl GarchModel {
    fn new(initial_annual_vol: f64) -> Self {
        // More aggressive parameters for crypto volatility
        let alpha = 0.12;  // Increased reaction to shocks
        let beta = 0.86;   // Slightly reduced persistence

        let seconds_per_year = 365.25 * 24.0 * 3600.0f64;
        let scaling_factor = seconds_per_year.sqrt();
        let initial_var = (initial_annual_vol / scaling_factor).powi(2);
        let omega = initial_var * (1.0 - alpha - beta) * 1.5;  // Boosted omega

        GarchModel {
            omega,
            alpha,
            beta,
            last_variance: initial_var,
            last_price: 0.0,
            long_run_variance: initial_var,
        }
    }

    fn update(&mut self, new_price: f64) -> f64 {
        let return_ = (new_price / self.last_price).ln();

        // More responsive update equation
        let shock = return_.powi(2).min(0.001);  // Cap large shocks
        self.last_variance = self.omega
            + self.alpha * shock
            + self.beta * self.last_variance;

        // Wider mean-reversion bounds (70-150%)
        self.last_variance = self.last_variance
            .max(self.long_run_variance * 0.7)
            .min(self.long_run_variance * 1.5);

        self.last_price = new_price;
        self.last_variance.sqrt()
    }

    fn annualized_vol(&self) -> f64 {
        let seconds_per_year = 365.25 * 24.0 * 3600.0f64;
        self.last_variance.sqrt() * seconds_per_year.sqrt() * 100.0
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {

    let hourly_data = get_data().await;
    let init_vol = compute_volatility(&hourly_data) / 100.0;
    println!("Initial annualized vol: {:.2}%", init_vol * 100.0);
    let mut garch = GarchModel::new(init_vol);

    // Get initial price
    let mut last_price = btc::fetch_spot().await?.result.index_price;
    garch.last_price = last_price;

    // Track volatility for debugging
    let mut vol_history = vec![];

    loop {
        let new_price = btc::fetch_spot().await?.result.index_price;

        // Update GARCH model
        garch.update(new_price);
        let annualized_vol = garch.annualized_vol();
        vol_history.push(annualized_vol);

        // Print every 10 updates to reduce noise
        if vol_history.len() % 10 == 0 {
            println!(
                "Price={:.2}, Annualized Vol={:.2}% ({} updates, avg={:.2}%)",
                new_price,
                annualized_vol,
                vol_history.len(),
                vol_history.iter().sum::<f64>() / vol_history.len() as f64
            );
        }

        last_price = new_price;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}