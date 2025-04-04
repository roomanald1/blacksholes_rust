use rand::{Rng};
use rand_distr::{ StandardNormal, Distribution};
use rand_distr::num_traits::real::Real;
use rayon::prelude::*;
use crate::option_pricer::OptionType;

pub fn monte_carlo(option_type: OptionType, spot: f64, risk_free_rate: f64, vol: f64, time_to_expiry: f64, strike: f64, number_of_sims: usize) -> f64 {
    let mut total_payoff = 0.0;

    let drift = (risk_free_rate - 0.5 * vol * vol) * time_to_expiry;
    let vol_scaled = vol * time_to_expiry.sqrt();
    total_payoff = (0..number_of_sims).into_par_iter().map(|_| {
        let mut rng = rand::rng();
        let random_shock: f64 = rng.sample(StandardNormal);
        let st = spot * f64::exp(drift + vol_scaled * random_shock);

        match option_type {
            OptionType::Call => {
                f64::max(st - strike, 0.0)
            },
            OptionType::Put => {
                f64::max(strike - st, 0.0)
            }
        }
    }).sum();

    let mean_payoff = total_payoff / number_of_sims as f64;
    let option_price = f64::exp(-risk_free_rate * time_to_expiry) * mean_payoff;
    option_price
}


#[cfg(test)]
mod test{
    use crate::montecarlo::{monte_carlo};
    use crate::option_pricer::{OptionPricer, OptionType};

    #[test]
    fn monte_carlo_vs_blacksholes() {
        let spot = 98.0;
        let risk_free_rate = 0.05;
        let vol = 0.1;
        let time_to_expiry = 0.5;
        let strike = 100.0;
        let num_sims = 1_000_000;

        let bs_start = std::time::Instant::now();
        let b_s = OptionPricer::new(spot, strike, time_to_expiry, risk_free_rate, vol)
            .calc_premium(OptionType::Call);
        let bs_duration = bs_start.elapsed();

        let mc_start = std::time::Instant::now();
        let m_c = monte_carlo(
            OptionType::Call,
            spot,
            risk_free_rate,
            vol,
            time_to_expiry,
            strike,
            num_sims,
        );
        let mc_duration = mc_start.elapsed();

        // Allow 1% difference or 0.1 absolute difference, whichever is larger
        let tolerance = f64::max(b_s * 0.01, 0.1);
        assert!(
            (b_s - m_c).abs() < tolerance,
            "Black-Scholes: {}, Monte Carlo: {} (diff: {}, tolerance: {})",
            b_s,
            m_c,
            (b_s - m_c).abs(),
            tolerance
        );

        println!("Black-Scholes: {}ms MonteCarlo: {}ms", bs_duration.as_millis(), mc_duration.as_millis());
    }
}