use std::cmp::PartialEq;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OptionType {
    Call, Put
}

#[derive(Debug, Clone)]
pub struct OptionValue {
    pub option_type: Option<OptionType>,
    pub premium: Option<f64>,
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub vega: Option<f64>,
    pub theta: Option<f64>,
}

pub struct OptionPricer {
    pub option_type: OptionType,
    pub spot_price: f64,
    pub strike_price: f64,
    pub time_to_expiration: f64,
    pub risk_free_rate: f64,
    pub volatility: f64,
    pub d1: f64,
    pub d2: f64,
    pub discount_factor: f64,
}


impl OptionPricer {
    pub fn new(
        option_type: OptionType,
        spot_price: f64,
        strike_price: f64,
        time_to_expiration: f64,
        risk_free_rate: f64,
        volatility: f64,
        ) -> Self {
        let d1 = calculate_d1(spot_price,strike_price, time_to_expiration, risk_free_rate, volatility);
        let d2 = calculate_d2(d1, volatility, time_to_expiration);
        let discount_factor = discount_factor(risk_free_rate, time_to_expiration);
        OptionPricer {
            option_type,
            spot_price,
            strike_price,
            time_to_expiration,
            risk_free_rate,
            volatility,
            d1,
            d2,
            discount_factor
        }
    }

    pub fn calc_premium(&self) -> f64 {
        match self.option_type {
            OptionType::Call => {
                // Call
                // -------------------------
                //
                // C= S × N(d₁) - Ke⁻ʳᵀ × N(d₂)
                self.spot_price * normal_cdf(self.d1) - self.strike_price * self.discount_factor * normal_cdf(self.d2)
            },
            OptionType::Put => {
                // Put
                // -------------------------
                //
                // P = Ke⁻ʳᵀ × N(-d₂) - S × N(-d₁)
                self.strike_price * self.discount_factor * normal_cdf(-self.d2) - self.spot_price * normal_cdf(-self.d1)
            }
        }
    }

    pub fn calc_delta(&self) -> f64{
        let delta = normal_cdf(self.d1);
        match self.option_type{
            OptionType::Call => {
                delta
            },
            OptionType::Put => {
                delta - 1.0
            }
        }
    }

    pub fn calc_gamma(&self) -> f64 {
        let pdf = normal_pdf(self.d1);
        pdf / (self.spot_price * self.time_to_expiration.sqrt() * self.volatility)
    }

    pub fn calc_theta(&self) -> f64 {
        if self.time_to_expiration <= 0.0 {
            return 0.0;
        }

        let pdf = normal_pdf(self.d1);
        let term1 = (-self.spot_price * pdf * self.volatility) / (2.0 * self.time_to_expiration.sqrt());


        match self.option_type {
            OptionType::Call => {
                let term2 = self.risk_free_rate * self.strike_price * self.discount_factor * normal_cdf(self.d2);
                term1 - term2
            },
            OptionType::Put => {
                let term2 = self.risk_free_rate * self.strike_price * self.discount_factor * normal_cdf(-self.d2);
                term1 + term2
            }
        }

    }

    pub fn calc_vega(&self) -> f64 {
        // Handle zero time case first
        if self.time_to_expiration <= 0.0 {
            return 0.0;
        }

        // Handle zero volatility case
        if self.volatility <= 0.0 {
            return 0.0;
        }
        let pdf = normal_pdf(self.d1);
        self.spot_price * self.time_to_expiration.sqrt() * pdf
    }

    pub fn calculate_option(&self) -> OptionValue {
        let premium = self.calc_premium();
        let delta = self.calc_delta();
        let gamma = self.calc_gamma();
        let vega = self.calc_vega();
        let theta = self.calc_theta();
        OptionValue { premium: Some(premium), delta: Some(delta), option_type: Some(self.option_type), gamma: Some(gamma), vega: Some(vega), theta: Some(theta) }
    }

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




/// DiscountFactor
/// -------------------------
///
/// DF = e⁻ʳᵀ
fn discount_factor(risk_free_rate:f64, time_to_expiration_in_years: f64) -> f64 {
    (-risk_free_rate * time_to_expiration_in_years).exp()
}

/// Cumulative Normal Distribution (N(x))
/// -------------------------
///
/// N(x) = 0.5 × [1 + erf(x/√2)]
#[inline]
fn normal_cdf(x:f64) -> f64 {
    (0.5) * (1.0 + libm::erf(x / std::f64::consts::SQRT_2))
}

/// Standard normal probability density function (PDF).
fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x.powi(2)).exp() / (2.0 * std::f64::consts::PI).sqrt()
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

        let call_price =  OptionPricer::new(OptionType::Call, s,k, t, r, sigma).calc_premium();
        // Expected value from verified calculator
        assert_abs_diff_eq!(call_price, 10.4506, epsilon = 1e-4);

        let call_price = OptionPricer::new(OptionType::Call, 150.0, 100.0, 1.0, 0.05, 0.2).calc_premium();
        // Test case 2: Deep ITM call
        assert_abs_diff_eq!(
            call_price,
            55.2417,
            epsilon = 0.3
        );
    }

    #[test]
    fn test_normal_pdf() {
        assert!((normal_pdf(0.0) - 0.398942).abs() < 1e-6);
        assert!((normal_pdf(1.0) - 0.241971).abs() < 1e-6);
    }

    #[test]
    fn test_black_scholes_put() {
        // Test case 1: ATM put
        assert_abs_diff_eq!(
            OptionPricer::new(OptionType::Put, 100.0, 100.0, 1.0, 0.05, 0.2).calc_premium(),
            5.5735,
            epsilon = 1e-4
        );

        // Test case 2: Deep OTM put
        let put_value = OptionPricer::new(OptionType::Put, 100.0, 150.0, 1.0, 0.05, 0.2).calc_premium();
        assert!(put_value > 40.0 && put_value < 45.0);
    }




    #[test]
    fn test_vega_at_the_money() {
        let spot = 100.0;
        let strike = 100.0;
        let time = 1.0;     // 1 year
        let rate = 0.05;    // 5%
        let vol = 0.2;      // 20%

        let vega = OptionPricer::new(OptionType::Call, spot, strike, time, rate, vol).calc_vega();

        assert_abs_diff_eq!(vega, 37.5165, epsilon = 0.01);
    }

    #[test]
    fn test_vega_volatility_independence() {
        let vega_low = OptionPricer::new(OptionType::Call, 100.0, 100.0, 1.0, 0.05, 0.2).calc_vega();
        let vega_high = OptionPricer::new(OptionType::Call, 100.0, 100.0, 1.0, 0.05, 0.5).calc_vega();
        assert_abs_diff_eq!(vega_low, vega_high, epsilon = 0.001);
    }

    #[test]
    fn test_vega_edge_cases() {
        // Zero time
        assert_eq!(OptionPricer::new(OptionType::Call, 100.0, 100.0, 0.0, 0.05, 0.2).calc_vega(), 0.0);

        // Zero volatility
        assert_eq!(OptionPricer::new(OptionType::Call, 100.0, 100.0, 1.0, 0.05, 0.0).calc_vega(), 0.0);

        // Zero spot price
        assert_eq!(OptionPricer::new(OptionType::Call, 0.0, 100.0, 1.0, 0.05, 0.5).calc_vega(), 0.0);

        // All zeros
        assert_eq!(OptionPricer::new(OptionType::Call, 0.0, 0.0, 0.0, 0.0, 0.0).calc_vega(), 0.0);
    }

    #[test]
    fn test_vega_extreme_spot() {
        // Deep ITM call (spot >> strike)
        let vega_itm = OptionPricer::new(OptionType::Call, 1000.0, 100.0, 1.0, 0.05, 0.2).calc_vega();
        // Deep OTM call (spot << strike)
        let vega_otm = OptionPricer::new(OptionType::Call, 10.0, 100.0, 1.0, 0.05, 0.2).calc_vega();
        assert!(vega_itm < 0.001 && vega_otm < 0.001); // Near-zero for extreme spots
    }



    #[test]
    fn test_theta_atm_call() {
        let theta = OptionPricer::new(OptionType::Call, 100.0, 100.0, 1.0, 0.05, 0.2).calc_theta();
        assert_abs_diff_eq!(theta, -6.414, epsilon = 0.03);
    }

    #[test]
    fn test_theta_itm_put() {
        let theta = OptionPricer::new(OptionType::Put, 90.0, 100.0, 0.5, 0.05, 0.3).calc_theta();
        assert_abs_diff_eq!(theta, -3.9674, epsilon = 0.03);  // Corrected expectation
    }

    #[test]
    fn test_theta_expired() {
        let theta = OptionPricer::new(OptionType::Call, 100.0, 100.0, 0.0, 0.05, 0.2).calc_theta();
        assert_eq!(theta, 0.0);
    }
}
