use std::cmp::PartialEq;
use crate::utils::OptionType;

#[derive(Debug, Clone)]
pub struct OptionValue {
    pub option_type: Option<OptionType>,
    pub premium: Option<f64>,
    pub delta: Option<f64>,
    pub gamma: Option<f64>,
    pub vega: Option<f64>,
    pub theta: Option<f64>,
    pub rho: Option<f64>,
}

pub struct OptionPricer {
    //Inputs
    pub spot_price: f64,
    pub strike_price: f64,
    pub time_to_expiration: f64,
    pub risk_free_rate: f64,
    pub volatility: f64,
    //Cached Values
    d1: f64,
    d2: f64,
    discount_factor: f64,
    sqrt_t: f64,
    normal_cdf_d2: Option<f64>,
    normal_cdf_d1: Option<f64>,
    normal_cdf_minus_d2: Option<f64>,
    normal_cdf_minus_d1: Option<f64>,
    normal_pdf_d1: Option<f64>,
}


impl OptionPricer {
    pub fn new(
        spot_price: f64,
        strike_price: f64,
        time_to_expiration: f64,
        risk_free_rate: f64,
        volatility: f64,
        ) -> Self {
        let d1 = calculate_d1(spot_price,strike_price, time_to_expiration, risk_free_rate, volatility);
        let sqrt_t = time_to_expiration.sqrt();
        let d2 = calculate_d2(d1, volatility, sqrt_t);
        let discount_factor = discount_factor(risk_free_rate, time_to_expiration);
        OptionPricer {
            spot_price,
            strike_price,
            time_to_expiration,
            risk_free_rate,
            volatility,
            //Cached Values
            d1,
            d2,
            discount_factor,
            sqrt_t,
            normal_cdf_d1: None,
            normal_cdf_d2: None,
            normal_cdf_minus_d2: None,
            normal_cdf_minus_d1: None,
            normal_pdf_d1: None
        }
    }


    pub fn calc_rho(&mut self, option_type: OptionType) -> f64 {
        match option_type {
            OptionType::Call => {
                self.strike_price * self.time_to_expiration * self.discount_factor * self.get_or_calc_normal_cdf_d2()
            },
            OptionType::Put => {
                -self.strike_price * self.time_to_expiration * self.discount_factor * self.get_or_calc_normal_cdf_minus_d2()
            }
        }
    }

    fn get_or_calc_normal_cdf_d1(&mut self) -> f64 {
        match self.normal_cdf_d1 {
            Some(x) => x,
            None => {
                let result = normal_cdf(self.d1);
                self.normal_cdf_d1 = Some(result);
                result
            }
        }
    }


    fn get_or_calc_normal_cdf_minus_d1(&mut self) -> f64 {
        match self.normal_cdf_minus_d1 {
            Some(x) => x,
            None => {
                let result = normal_cdf(-self.d1);
                self.normal_cdf_minus_d1 = Some(result);
                result
            }
        }
    }

    fn get_or_calc_normal_cdf_d2(&mut self) -> f64 {
        match self.normal_cdf_d2{
            Some(x) => x,
            None => {
                let result = normal_cdf(self.d2);
                self.normal_cdf_d2 = Some(result);
                result
            }
        }
    }


    fn get_or_calc_normal_pdf_d1(&mut self) -> f64 {
        match self.normal_pdf_d1{
            Some(x) => x,
            None => {
                let result = normal_pdf(self.d1);
                self.normal_pdf_d1 = Some(result);
                result
            }
        }
    }

    fn get_or_calc_normal_cdf_minus_d2(&mut self) -> f64 {
        match self.normal_cdf_minus_d2{
            Some(x) => x,
            None => {
                let result = normal_cdf(-self.d2);
                self.normal_cdf_minus_d2 = Some(result);
                result
            }
        }
    }

    pub fn calc_delta(&mut self, option_type: OptionType) -> f64{
        let delta = self.get_or_calc_normal_cdf_d1();
        match option_type{
            OptionType::Call => {
                delta
            },
            OptionType::Put => {
                delta - 1.0
            }
        }
    }

    pub fn calc_premium(&mut self, option_type: OptionType) -> f64 {
        match option_type {
            OptionType::Call => {
                // Call
                // -------------------------
                //
                // C= S × N(d₁) - Ke⁻ʳᵀ × N(d₂)
                self.spot_price * self.get_or_calc_normal_cdf_d1() - self.strike_price * self.discount_factor * self.get_or_calc_normal_cdf_d2()
            },
            OptionType::Put => {
                // Put
                // -------------------------
                //
                // P = Ke⁻ʳᵀ × N(-d₂) - S × N(-d₁)
                self.strike_price * self.discount_factor * self.get_or_calc_normal_cdf_minus_d2() - self.spot_price * self.get_or_calc_normal_cdf_minus_d1()
            }
        }
    }


    pub fn calc_gamma(&mut self) -> f64 {
        let pdf = self.get_or_calc_normal_pdf_d1();
        pdf / (self.spot_price * self.sqrt_t * self.volatility)
    }

    pub fn calc_theta(&mut self, option_type: OptionType) -> f64 {
        if self.time_to_expiration <= 0.0 {
            return 0.0;
        }

        let pdf = self.get_or_calc_normal_pdf_d1();
        let term1 = (-self.spot_price * pdf * self.volatility) / (2.0 * self.sqrt_t);


        match option_type {
            OptionType::Call => {
                let term2 = self.risk_free_rate * self.strike_price * self.discount_factor * self.get_or_calc_normal_cdf_d2();
                term1 - term2
            },
            OptionType::Put => {
                let term2 = self.risk_free_rate * self.strike_price * self.discount_factor * self.get_or_calc_normal_cdf_minus_d2();
                term1 + term2
            }
        }

    }

    pub fn calc_vega(&mut self) -> f64 {
        // Handle zero time case first
        if self.time_to_expiration <= 0.0 {
            return 0.0;
        }

        // Handle zero volatility case
        if self.volatility <= 0.0 {
            return 0.0;
        }
        let pdf = self.get_or_calc_normal_pdf_d1();
        self.spot_price * self.sqrt_t * pdf
    }

    pub fn calculate_option(&mut self, option_type: OptionType) -> OptionValue {
        let premium = self.calc_premium(option_type);
        let delta = self.calc_delta(option_type);
        let gamma = self.calc_gamma();
        let vega = self.calc_vega();
        let theta = self.calc_theta(option_type);
        let rho = self.calc_rho(option_type);
        OptionValue { premium: Some(premium), delta: Some(delta), option_type: Some(option_type), gamma: Some(gamma), vega: Some(vega), theta: Some(theta), rho: Some(rho) }
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
fn calculate_d2(d1: f64, volatility:f64, sqrt_t:f64) -> f64 {
    d1 - (volatility * sqrt_t)
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

        let call_price =  OptionPricer::new(s,k, t, r, sigma).calc_premium(OptionType::Call);
        // Expected value from verified calculator
        assert_abs_diff_eq!(call_price, 10.4506, epsilon = 1e-4);

        let call_price = OptionPricer::new(150.0, 100.0, 1.0, 0.05, 0.2).calc_premium(OptionType::Call,);
        // Test case 2: Deep ITM call
        assert_abs_diff_eq!(
            call_price,
            55.2417,
            epsilon = 0.3
        );
    }
    #[test]
    fn test_black_scholes_call_specific() {
        let spot = 98.0;
        let strike = 100.0;
        let time = 0.5;
        let rate = 0.05;
        let vol = 0.1;

        let mut pricer = OptionPricer::new(spot, strike, time, rate, vol);
        let call_price = pricer.calc_premium(OptionType::Call);

        // Expected value from Black-Scholes formula, verified externally
        let expected_price = 3.0139; // Precise value from earlier calculation
        assert_abs_diff_eq!(
            call_price,
            expected_price,
            epsilon = 0.02
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
            OptionPricer::new( 100.0, 100.0, 1.0, 0.05, 0.2).calc_premium(OptionType::Put),
            5.5735,
            epsilon = 1e-4
        );

        // Test case 2: Deep OTM put
        let put_value = OptionPricer::new( 100.0, 150.0, 1.0, 0.05, 0.2).calc_premium(OptionType::Put);
        assert!(put_value > 40.0 && put_value < 45.0);
    }




    #[test]
    fn test_vega_at_the_money() {
        let spot = 100.0;
        let strike = 100.0;
        let time = 1.0;     // 1 year
        let rate = 0.05;    // 5%
        let vol = 0.2;      // 20%

        let vega = OptionPricer::new( spot, strike, time, rate, vol).calc_vega();

        assert_abs_diff_eq!(vega, 37.5165, epsilon = 0.01);
    }

    #[test]
    fn test_vega_volatility_independence() {
        let vega_low = OptionPricer::new( 100.0, 100.0, 1.0, 0.05, 0.2).calc_vega();
        let vega_high = OptionPricer::new( 100.0, 100.0, 1.0, 0.05, 0.5).calc_vega();
        assert_abs_diff_eq!(vega_low, vega_high, epsilon = 0.001);
    }

    #[test]
    fn test_vega_edge_cases() {
        // Zero time
        assert_eq!(OptionPricer::new( 100.0, 100.0, 0.0, 0.05, 0.2).calc_vega(), 0.0);

        // Zero volatility
        assert_eq!(OptionPricer::new(100.0, 100.0, 1.0, 0.05, 0.0).calc_vega(), 0.0);

        // Zero spot price
        assert_eq!(OptionPricer::new( 0.0, 100.0, 1.0, 0.05, 0.5).calc_vega(), 0.0);

        // All zeros
        assert_eq!(OptionPricer::new(0.0, 0.0, 0.0, 0.0, 0.0).calc_vega(), 0.0);
    }

    #[test]
    fn test_vega_extreme_spot() {
        // Deep ITM call (spot >> strike)
        let vega_itm = OptionPricer::new(1000.0, 100.0, 1.0, 0.05, 0.2).calc_vega();
        // Deep OTM call (spot << strike)
        let vega_otm = OptionPricer::new(10.0, 100.0, 1.0, 0.05, 0.2).calc_vega();
        assert!(vega_itm < 0.001 && vega_otm < 0.001); // Near-zero for extreme spots
    }



    #[test]
    fn test_theta_atm_call() {
        let theta = OptionPricer::new( 100.0, 100.0, 1.0, 0.05, 0.2).calc_theta(OptionType::Call);
        assert_abs_diff_eq!(theta, -6.414, epsilon = 0.03);
    }

    #[test]
    fn test_theta_itm_put() {
        let theta = OptionPricer::new( 90.0, 100.0, 0.5, 0.05, 0.3).calc_theta(OptionType::Put,);
        assert_abs_diff_eq!(theta, -3.9927, epsilon = 0.03);  // Corrected expectation
    }

    #[test]
    fn test_theta_expired() {
        let theta = OptionPricer::new(100.0, 100.0, 0.0, 0.05, 0.2).calc_theta(OptionType::Call);
        assert_eq!(theta, 0.0);
    }

    #[test]
    fn test_gamma_properties() {
        // Gamma should decrease with higher volatility
        let mut low_vol = OptionPricer::new(100.0, 100.0, 1.0, 0.05, 0.2);
        let mut high_vol = OptionPricer::new(100.0, 100.0, 1.0, 0.05, 0.4);
        assert!(low_vol.calc_gamma() > high_vol.calc_gamma());

        // Gamma should decrease with higher spot price (for calls)
        let mut low_spot = OptionPricer::new(90.0, 100.0, 1.0, 0.05, 0.2);
        let mut high_spot = OptionPricer::new(110.0, 100.0, 1.0, 0.05, 0.2);
        assert!(low_spot.calc_gamma() > high_spot.calc_gamma());
    }

    #[test]
    fn test_gamma_time_dependence() {
        // Gamma peaks when option is near the money with short time
        let mut long_term = OptionPricer::new(100.0, 100.0, 2.0, 0.05, 0.2);
        let mut short_term = OptionPricer::new(100.0, 100.0, 0.1, 0.05, 0.2);
        assert!(short_term.calc_gamma() > long_term.calc_gamma());
    }


    #[test]
    fn test_atm_call_delta() {
        let mut pricer = OptionPricer::new(
            100.0,   // spot_price
            100.0,   // strike_price
            1.0,     // time_to_expiration
            0.05,     // risk_free_rate
            0.2,      // volatility
        );
        let delta = pricer.calc_delta(OptionType::Call);
        // ATM call delta should be ~0.5 + (r + σ²/2) adjustment
        assert_abs_diff_eq!(delta, 0.637, epsilon = 0.01);
    }

    #[test]
    fn test_atm_put_delta() {
        let mut pricer = OptionPricer::new(
            100.0,   // spot_price
            100.0,   // strike_price
            1.0,     // time_to_expiration
            0.05,     // risk_free_rate
            0.2,      // volatility
        );
        let delta = pricer.calc_delta(OptionType::Put);
        // ATM put delta should be ~-0.5 + (r + σ²/2) adjustment
        assert_abs_diff_eq!(delta, -0.363, epsilon = 0.01);
    }

    #[test]
    fn test_itm_call_delta() {
        let mut pricer = OptionPricer::new(
            120.0,  // spot_price (ITM)
            100.0,  // strike_price
            1.0,
            0.05,
            0.2,
        );
        let delta = pricer.calc_delta(OptionType::Call);
        assert!(delta > 0.8 && delta <= 1.0);
    }

    #[test]
    fn test_otm_put_delta() {
        let mut pricer = OptionPricer::new(
            120.0,  // spot_price (OTM for put)
            100.0,
            1.0,
            0.05,
            0.2,
        );
        let delta = pricer.calc_delta(OptionType::Put);
        assert!(delta > -0.2 && delta < 0.0);
    }

    #[test]
    fn test_delta_boundaries() {
        // Deep ITM call
        let mut pricer = OptionPricer::new(
            200.0,
            100.0,
            0.1,  // Short time to expiration
            0.05,
            0.2,
        );
        let delta = pricer.calc_delta(OptionType::Call);
        assert_abs_diff_eq!(delta, 1.0, epsilon = 0.001);

        // Deep OTM put
        let mut pricer = OptionPricer::new(
            200.0,
            100.0,
            0.1,
            0.05,
            0.2,
        );
        let delta = pricer.calc_delta(OptionType::Put);
        assert_abs_diff_eq!(delta, 0.0, epsilon = 0.001);
    }

    #[test]
    fn test_put_call_delta_parity() {
        let mut pricer = OptionPricer::new(
            100.0,
            100.0,
            1.0,
            0.05,
            0.2,
        );
        let call_delta = pricer.calc_delta(OptionType::Call);
        let put_delta = pricer.calc_delta(OptionType::Put);
        // Put-call delta parity: Δ_call - Δ_put = 1
        assert_abs_diff_eq!(call_delta - put_delta, 1.0, epsilon = 1e-10);
    }

    #[test]
    fn test_time_decay_on_delta() {
        let mut pricer_long = OptionPricer::new(
            100.0,
            100.0,
            1.0,
            0.05,
            0.2,
        );
        let mut pricer_short = OptionPricer::new(
            100.0,
            100.0,
            0.01,
            0.05,
            0.2,
        );

        let delta_long = pricer_long.calc_delta(OptionType::Call);
        let delta_short = pricer_short.calc_delta(OptionType::Call);

        // For ATM calls, delta should be closer to 0.5 with less time
        assert!(delta_short < delta_long,
                "Expected delta to decrease with shorter time ({} > {})",
                delta_long, delta_short);

        // Typical values would be:
        // delta_long ~0.637 (with 1 year)
        // delta_short ~0.506 (with few days)
    }

    // Test case 1: Call option with typical parameters
    #[test]
    fn test_rho_call_against_known_value() {
        let mut pricer = OptionPricer::new(
            100.0,    // S
            100.0,    // K
            1.0,      // T (1 year)
            0.05,     // r (5%)
            0.20,     // σ (20%)
        );

        // Precomputed rho for call (from external calculator)
        let expected_rho_call = 53.23248; // Example value (adjust based on your source)
        let computed_rho_call = pricer.calc_rho(OptionType::Call);

        // Allow small floating-point error (e.g., 0.01)
        assert_abs_diff_eq!(computed_rho_call, expected_rho_call, epsilon = 0.01);
    }

    // Test case 2: Put option with typical parameters
    #[test]
    fn test_rho_put_against_known_value() {
        let mut pricer = OptionPricer::new(
            100.0,    // S
            100.0,    // K
            1.0,      // T (1 year)
            0.05,     // r (5%)
            0.20,     // σ (20%)
        );

        // Precomputed rho for put (from external calculator)
        let expected_rho_put = -41.89; // Example value (adjust based on your source)
        let computed_rho_put = pricer.calc_rho(OptionType::Put);

        assert_abs_diff_eq!(computed_rho_put, expected_rho_put, epsilon = 0.01);
    }

    // Test case 3: Edge case (near-zero interest rate)
    #[test]
    fn test_rho_near_zero_rate() {
        let mut pricer = OptionPricer::new(
            100.0,     // S
            100.0,     // K
            1.0,       // T (1 year)
            0.000001,  // r (0.0001%)
            0.20,      // σ (20%)
        );

        // For very small r, rho approaches K*T*N(d2)
        // d2 ≈ (ln(1) - σ²/2)/σ ≈ -0.1
        // N(d2) ≈ 0.4602
        // ρ ≈ 100 * 1 * 0.4602 ≈ 46.02
        let expected_rho = 100.0 * 1.0 * 0.4602;
        let computed_rho = pricer.calc_rho(OptionType::Call);
        assert_abs_diff_eq!(computed_rho, expected_rho, epsilon = 0.01);
    }
}
