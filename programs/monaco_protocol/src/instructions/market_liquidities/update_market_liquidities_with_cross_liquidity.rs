use crate::instructions::market_liquidities::calculate_cross_price::CrossPrice;
use crate::state::market_liquidities::{LiquidityKey, MarketLiquidities};
use anchor_lang::prelude::*;

pub fn update_market_liquidities_with_cross_liquidity(
    market_liquidities: &mut MarketLiquidities,
    for_outcome: bool,
    cross_liquidity: LiquidityKey,
    source_liquidities: Vec<LiquidityKey>,
) -> Result<()> {
    // calculate price based on provided
    let mut cross_price_calculator = CrossPrice::new(source_liquidities.len());
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
                        if for_outcome {
                            market_liquidities
                                .get_liquidity_against(
                                    source_liquidity.outcome,
                                    source_liquidity.price,
                                )
                                .map(|source_liquidity| source_liquidity.liquidity)
                                .unwrap_or(0_u64)
                        } else {
                            market_liquidities
                                .get_liquidity_for(source_liquidity.outcome, source_liquidity.price)
                                .map(|source_liquidity| source_liquidity.liquidity)
                                .unwrap_or(0_u64)
                        }
                    })
                    .min()
                    .unwrap_or(0_u64);

                // update liquidity
                if for_outcome {
                    market_liquidities.set_liquidity_for(
                        cross_liquidity.outcome,
                        cross_liquidity.price,
                        cross_liquidity_stake,
                    );
                } else {
                    market_liquidities.set_liquidity_against(
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
