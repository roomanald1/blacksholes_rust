
# BlackSholes Option Pricing

## Call
C = S × N(d₁) - Ke⁻ʳᵀ × N(d₂)

## Put
P = Ke⁻ʳᵀ × N(-d₂) - S × N(-d₁)

Where
N(x) = Cumulative standard normal distribution function
ln = Natural logarithm
e⁻ʳᵀ = Discount factor (exp(-r × T))

Cumulative Normal Distribution (N(x))

 -> N(x) = 0.5 × [1 + erf(x/√2)]

Where erf is the error function.


## Example Calculation (ATM Call)
Given:

S = 100, K = 100, T = 1, r = 0.05, σ = 0.2


d₁ = [ln(100/100) + (0.05 + 0.2²/2)×1] / (0.2×√1) = 0.325
d₂ = 0.325 - 0.2×√1 = 0.125
N(d₁) = 0.627
N(d₂) = 0.550
C = 100×0.627 - 100×e⁻⁰·⁰⁵×0.550 ≈ 10.45