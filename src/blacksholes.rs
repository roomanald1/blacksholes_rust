use libm::erf;

/// Cumulative Normal Distribution (N(x))
/// -------------------------
///
/// N(x) = 0.5 × [1 + erf(x/√2)]
#[inline]
fn normal_cdf(x:f64) -> f64 {
   (0.5) * (1.0 + erf(x / std::f64::consts::SQRT_2))
}



/// D1
/// -------------------------
///
/// d₁ = [ln(S/K) + (r + σ²/2)T] / (σ√T)
fn calculate_d1(spot_price:f64, strike_price:f64, time_to_expiration_in_years:f64, risk_free_rate:f64, volatility: f64) -> f64 {
    ((spot_price / strike_price).ln() + (risk_free_rate+ ((volatility * volatility)/2.0)) * time_to_expiration_in_years) / (volatility * time_to_expiration_in_years.sqrt())
}

/// D2
/// -------------------------
///
/// d₂ = d₁ - σ√T
fn calculate_d2(d1: f64, volatility:f64, time_to_expiration_in_years:f64) -> f64 {
    d1 - (volatility * time_to_expiration_in_years.sqrt())
}


/// Call
/// -------------------------
///
/// C= S × N(d₁) - Ke⁻ʳᵀ × N(d₂)
pub fn calc_call(spot_price:f64, strike_price:f64, time_to_expiration_in_years:f64, risk_free_rate:f64, volatility: f64) -> f64 {
    let d1 = calculate_d1(spot_price, strike_price, time_to_expiration_in_years, risk_free_rate, volatility);
    let d2 = calculate_d2(d1, volatility, time_to_expiration_in_years);
    let discount_factor = (-risk_free_rate * time_to_expiration_in_years).exp();
    spot_price * normal_cdf(d1) - strike_price * discount_factor * normal_cdf(d2)
}


/// DiscountFactor
/// -------------------------
///
/// DF = e⁻ʳᵀ
fn discount_factor(risk_free_rate:f64, time_to_expiration_in_years: f64) -> f64 {
    (-risk_free_rate * time_to_expiration_in_years).exp()
}

/// Put
/// -------------------------
///
/// P = Ke⁻ʳᵀ × N(-d₂) - S × N(-d₁)
pub fn calc_put(spot_price:f64, strike_price:f64, time_to_expiration_in_years:f64, risk_free_rate:f64, volatility: f64) -> f64 {
    let d1 = calculate_d1(spot_price, strike_price, time_to_expiration_in_years, risk_free_rate, volatility);
    let d2 = calculate_d2(d1, volatility, time_to_expiration_in_years);
    let discount_factor = discount_factor(risk_free_rate, time_to_expiration_in_years);
    strike_price * discount_factor * normal_cdf(-d2) - spot_price * normal_cdf(-d1)
}


#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq; // For floating-point comparisons

    #[test]
    fn test_normal_cdf() {
        // Test known values of N(x)
        assert_abs_diff_eq!(normal_cdf(0.0), 0.5, epsilon = 1e-6);
        assert_abs_diff_eq!(normal_cdf(1.96), 0.975, epsilon = 1e-3);
        assert_abs_diff_eq!(normal_cdf(-1.96), 0.025, epsilon = 1e-3);
    }

    #[test]
    fn test_black_scholes_call() {
        // Test case 1: ATM option (S = K)
        let s = 100.0;
        let k = 100.0;
        let t = 1.0;
        let r = 0.05;
        let sigma = 0.2;

        let call_price = calc_call(s, k, t, r, sigma);
        // Expected value from verified calculator
        assert_abs_diff_eq!(call_price, 10.4506, epsilon = 1e-4);

        // Test case 2: Deep ITM call
        assert_abs_diff_eq!(
    calc_call(150.0, 100.0, 1.0, 0.05, 0.2),
    55.2417,
    epsilon = 0.3
);
    }

    #[test]
    fn test_black_scholes_put() {
        // Test case 1: ATM put
        assert_abs_diff_eq!(
        calc_put(100.0, 100.0, 1.0, 0.05, 0.2),
        5.5735,
        epsilon = 1e-4
    );

        // Test case 2: Deep OTM put
        let put_value = calc_put(100.0, 150.0, 1.0, 0.05, 0.2);
        assert!(put_value > 40.0 && put_value < 45.0);
    }
}
