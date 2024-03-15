use crate::instructions::calculate_stake_cross;
use crate::instructions::market_liquidities::calculate_cross_price::CrossPriceCalculator;
use crate::state::market_liquidities::{LiquidityKey, MarketLiquidities};
use anchor_lang::prelude::*;

pub fn update_market_liquidities_with_cross_liquidity(
    market_liquidities: &mut MarketLiquidities,
    source_for_outcome: bool,
    source_liquidities: Vec<LiquidityKey>,
    cross_liquidity: LiquidityKey,
) -> Result<()> {
    // calculate price based on provided
    let mut cross_price_calculator = CrossPriceCalculator::new(source_liquidities.len() + 1);
    source_liquidities
        .iter()
        .for_each(|source_liquidity| cross_price_calculator.add(source_liquidity.price));

    match cross_price_calculator.result() {
        Some(cross_price) => {
            // provided cross_liquidity.price is valid
            if cross_price == cross_liquidity.price {
                // calculate stake
                let cross_liquidity_stake = source_liquidities
                    .iter()
                    .map(|source_liquidity| {
                        if source_for_outcome {
                            calculate_stake_cross(
                                market_liquidities
                                    .get_liquidity_for(
                                        source_liquidity.outcome,
                                        source_liquidity.price,
                                    )
                                    .map(|source_liquidity| source_liquidity.liquidity)
                                    .unwrap_or(0_u64),
                                source_liquidity.price,
                                cross_price,
                            )
                        } else {
                            calculate_stake_cross(
                                market_liquidities
                                    .get_liquidity_against(
                                        source_liquidity.outcome,
                                        source_liquidity.price,
                                    )
                                    .map(|source_liquidity| source_liquidity.liquidity)
                                    .unwrap_or(0_u64),
                                source_liquidity.price,
                                cross_price,
                            )
                        }
                    })
                    .min()
                    .unwrap_or(0_u64);

                // update liquidity
                if source_for_outcome {
                    market_liquidities.set_liquidity_against(
                        cross_liquidity.outcome,
                        cross_liquidity.price,
                        cross_liquidity_stake,
                    );
                } else {
                    market_liquidities.set_liquidity_for(
                        cross_liquidity.outcome,
                        cross_liquidity.price,
                        cross_liquidity_stake,
                    );
                }
            } else {
                // TODO should we return error?
            }
        }
        None => {
            // TODO should we return error?
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::state::market_liquidities::{mock_market_liquidities, MarketOutcomePriceLiquidity};

    #[test]
    fn test_2_way_market() {
        let mut market_liquidities = mock_market_liquidities(Pubkey::new_unique());
        market_liquidities.add_liquidity_for(0, 3.0, 100).unwrap();
        market_liquidities.add_liquidity_for(0, 3.5, 100).unwrap();
        market_liquidities.add_liquidity_for(0, 4.125, 100).unwrap();

        assert_eq!(
            vec!((0, 3.0, 100), (0, 3.5, 100), (0, 4.125, 100)),
            liquidities(&market_liquidities.liquidities_for)
        );

        //------------------------------------------------------------------------------------------

        update_market_liquidities_with_cross_liquidity(
            &mut market_liquidities,
            true,
            vec![LiquidityKey::new(0, 3.0)],
            LiquidityKey::new(1, 1.5),
        )
        .expect("update_market_liquidities_with_cross_liquidity failed");

        update_market_liquidities_with_cross_liquidity(
            &mut market_liquidities,
            true,
            vec![LiquidityKey::new(0, 3.5)],
            LiquidityKey::new(1, 1.4),
        )
        .expect("update_market_liquidities_with_cross_liquidity failed");

        update_market_liquidities_with_cross_liquidity(
            &mut market_liquidities,
            true,
            vec![LiquidityKey::new(0, 4.125)],
            LiquidityKey::new(1, 1.32),
        )
        .expect("update_market_liquidities_with_cross_liquidity failed");

        assert_eq!(
            vec!((1, 1.5, 200), (1, 1.4, 250), (1, 1.32, 312)), // TODO deal with rounding on the last result
            liquidities(&market_liquidities.liquidities_against)
        );
    }

    #[test]
    fn test_3_way_market() {
        // 2.0, 3.0, 6.0
        // 2.1, 3.0, 5.25
        let mut market_liquidities = mock_market_liquidities(Pubkey::new_unique());
        market_liquidities.add_liquidity_for(0, 2.0, 100).unwrap();
        market_liquidities.add_liquidity_for(0, 2.1, 100).unwrap();
        market_liquidities.add_liquidity_for(1, 3.0, 100).unwrap();

        assert_eq!(
            vec!((0, 2.0, 100), (0, 2.1, 100), (1, 3.0, 100)),
            liquidities(&market_liquidities.liquidities_for)
        );

        //------------------------------------------------------------------------------------------

        update_market_liquidities_with_cross_liquidity(
            &mut market_liquidities,
            true,
            vec![LiquidityKey::new(0, 2.1), LiquidityKey::new(1, 3.0)],
            LiquidityKey::new(2, 5.25),
        )
        .expect("update_market_liquidities_with_cross_liquidity failed");
        update_market_liquidities_with_cross_liquidity(
            &mut market_liquidities,
            true,
            vec![LiquidityKey::new(0, 2.0), LiquidityKey::new(1, 3.0)],
            LiquidityKey::new(2, 6.0),
        )
        .expect("update_market_liquidities_with_cross_liquidity failed");

        assert_eq!(
            vec!((2, 6.0, 33), (2, 5.25, 40)), // TODO deal with rounding on the first result
            liquidities(&market_liquidities.liquidities_against)
        );
    }

    fn liquidities(liquidities: &Vec<MarketOutcomePriceLiquidity>) -> Vec<(u16, f64, u64)> {
        liquidities
            .iter()
            .map(|v| (v.outcome, v.price, v.liquidity))
            .collect::<Vec<(u16, f64, u64)>>()
    }
}
