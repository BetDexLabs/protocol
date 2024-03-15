use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::ops::{Div, Mul, Sub};

use crate::instructions::price_to_decimal;

/*
    price_a = price_b / (price_b - 1)
    price_a = price_bc / (price_bc - price_b - price_c)
    price_a = price_bcd / (price_bcd - price_bc - price_cd - price_bd)
    price_a = price_bcde / (price_bcde - price_bcd - price_bce - price_bde - price_cde)
*/
#[derive(Debug, Default)]
pub struct CrossPriceCalculator {
    pub full: Decimal,
    pub partials: Vec<Decimal>,
    pub partials_index: usize,
}

impl CrossPriceCalculator {
    pub fn new(size: usize) -> CrossPriceCalculator {
        CrossPriceCalculator {
            full: Decimal::ONE,
            partials: vec![Decimal::ONE; size - 1_usize],
            partials_index: size - 1_usize,
        }
    }

    pub fn add(&mut self, price: f64) {
        self.partials_index -= 1_usize; // this will overflow if method called too many times

        let price_decimal = price_to_decimal(price);

        self.full = self.full.mul(price_decimal);
        for (index, partial) in self.partials.iter_mut().enumerate() {
            if index != self.partials_index {
                *partial = partial.mul(price_decimal);
            }
        }
    }

    pub fn result(&self) -> Option<f64> {
        let mut sub;

        // 2-way market goes differently
        if self.partials.is_empty() {
            sub = self.full.sub(&Decimal::ONE);
        } else {
            sub = self.full;
            for partial in self.partials.iter() {
                sub = sub.sub(partial);
            }
        }

        let result = self.full.div(sub);
        let result_truncated = result.trunc_with_scale(3);

        if result.ne(&result_truncated) {
            None // it needs to fit in 3 decimals
        } else {
            result_truncated.to_f64()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_cross_price_2way() {
        let market_outcomes_count = 2;
        let mut cross_price = CrossPriceCalculator::new(market_outcomes_count);

        cross_price.add(3.0_f64);
        assert_eq!(1.5_f64, cross_price.result().unwrap());
    }

    #[test]
    fn test_cross_price_3way() {
        let market_outcomes_count = 3;
        let mut cross_price = CrossPriceCalculator::new(market_outcomes_count);

        cross_price.add(2.0_f64);
        cross_price.add(3.0_f64);
        assert_eq!(6.0_f64, cross_price.result().unwrap());
    }

    #[test]
    fn test_cross_price_4way() {
        let market_outcomes_count = 4;

        let mut cross_price1 = CrossPriceCalculator::new(market_outcomes_count);
        cross_price1.add(4.0_f64);
        cross_price1.add(4.0_f64);
        cross_price1.add(4.0_f64);
        assert_eq!(4.0_f64, cross_price1.result().unwrap());

        let mut cross_price2 = CrossPriceCalculator::new(market_outcomes_count);
        cross_price2.add(4.0_f64);
        cross_price2.add(4.0_f64);
        cross_price2.add(5.0_f64);
        assert_eq!(None, cross_price2.result());
    }
}
