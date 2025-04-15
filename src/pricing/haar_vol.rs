use std::collections::VecDeque;

pub struct HaarVolatility {
    prices: VecDeque<f64>,
    window_size: usize,
    scales: Vec<usize>,      // Analysis scales (e.g., [1, 5, 15, 60] for seconds)
    detail_energies: Vec<f64>, // Energy at each scale
}

impl HaarVolatility {
    pub fn new(window_size: usize, scales: Vec<usize>) -> Self {
        Self {
            prices: VecDeque::with_capacity(window_size),
            window_size,
            scales: scales.clone(),
            detail_energies: vec![0.0; scales.len()],
        }
    }

    /// Update with new price and return current volatility estimate
    pub fn update(&mut self, price: f64) -> f64 {
        self.prices.push_back(price);
        if self.prices.len() > self.window_size {
            self.prices.pop_front();
        }

        self.calculate_wavelet_energy();
        self.combined_volatility()
    }

    /// Core Haar transform implementation
    fn calculate_wavelet_energy(&mut self) {
        for (i, &scale) in self.scales.iter().enumerate() {
            if self.prices.len() < scale * 2 {
                self.detail_energies[i] = 0.0;
                continue;
            }

            let mut sum_sq_diff = 0.0;
            let mut count = 0;

            for chunk in self.prices.as_slices().0.windows(scale * 2).step_by(scale) {
                let (prev, curr) = chunk.split_at(scale);
                let avg_prev: f64 = prev.iter().sum::<f64>() / scale as f64;
                let avg_curr: f64 = curr.iter().sum::<f64>() / scale as f64;

                // Haar detail coefficient
                let diff = avg_prev - avg_curr;
                sum_sq_diff += diff * diff;
                count += 1;
            }

            self.detail_energies[i] = (sum_sq_diff / count as f64).sqrt();
        }
    }

    /// Combine scales using root-mean-square
    fn combined_volatility(&self) -> f64 {
        let sum_energy: f64 = self.detail_energies.iter()
            .enumerate()
            .map(|(i, &energy)| {
                let weight = 1.0 / (self.scales[i] as f64).sqrt();
                energy * weight
            })
            .sum::<f64>();

        // Annualize (assuming 1-second data)
        sum_energy * (86400.0f64 * 365.0f64).sqrt()
    }

    /// Get volatility breakdown by time scale
    pub fn scale_volatilities(&self) -> Vec<(usize, f64)> {
        self.scales.iter()
            .zip(&self.detail_energies)
            .map(|(&scale, &vol)| (scale, vol * 100.0)) // as percentage
            .collect()
    }
}