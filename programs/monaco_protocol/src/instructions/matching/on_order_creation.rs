use anchor_lang::prelude::*;

use crate::error::CoreError;
use crate::error::CoreError::MatchingQueueIsFull;
use crate::instructions::calculate_stake_cross;
use crate::state::market_liquidities::MarketLiquidities;
use crate::state::market_matching_queue_account::*;
use crate::state::order_account::*;

#[cfg(test)]
use crate::state::market_liquidities::MarketOutcomePriceLiquidity;

pub const MATCH_CAPACITY: usize = 10_usize; // an arbitrary number

pub fn on_order_creation(
    market_liquidities: &mut MarketLiquidities,
    market_matching_queue: &mut MarketMatchingQueue,
    order_pk: &Pubkey,
    order: &mut Order,
) -> Result<Vec<(u64, f64)>> {
    match order.for_outcome {
        true => match_for_order(market_liquidities, market_matching_queue, order_pk, order),
        false => match_against_order(market_liquidities, market_matching_queue, order_pk, order),
    }
}

fn match_for_order(
    market_liquidities: &mut MarketLiquidities,
    market_matching_queue: &mut MarketMatchingQueue,
    order_pk: &Pubkey,
    order: &mut Order,
) -> Result<Vec<(u64, f64)>> {
    let mut order_matches = Vec::with_capacity(MATCH_CAPACITY);
    let order_outcome = order.market_outcome_index;

    // FOR order matches AGAINST liquidity
    let liquidities = &market_liquidities.liquidities_against;

    for liquidity in liquidities
        .iter()
        .filter(|element| element.outcome == order_outcome)
    {
        if order.stake_unmatched == 0_u64 {
            break; // no need to loop any further
        }
        if order_matches.len() == order_matches.capacity() {
            break; // can't loop any further
        }
        if liquidity.price < order.expected_price {
            break; // liquidity.price >= expected_price must be true
        }

        let stake_matched = liquidity.liquidity.min(order.stake_unmatched);
        if liquidity.sources.is_empty() {
            // straight match
            market_matching_queue
                .matches
                .enqueue(OrderMatch::maker(
                    !order.for_outcome,
                    order.market_outcome_index,
                    liquidity.price,
                    stake_matched,
                ))
                .ok_or(MatchingQueueIsFull)?;
        } else {
            // cross match
            for liquidity_source in &liquidity.sources {
                let liquidity_source_stake_matched =
                    calculate_stake_cross(stake_matched, liquidity.price, liquidity_source.price);
                market_matching_queue
                    .matches
                    .enqueue(OrderMatch::maker(
                        order.for_outcome,
                        liquidity_source.outcome,
                        liquidity_source.price,
                        liquidity_source_stake_matched,
                    ))
                    .ok_or(MatchingQueueIsFull)?;
            }
        }

        // record taker match
        market_matching_queue
            .matches
            .enqueue(OrderMatch::taker(
                *order_pk,
                order.for_outcome,
                order.market_outcome_index,
                liquidity.price,
                stake_matched,
            ))
            .ok_or(MatchingQueueIsFull)?;

        // this needs to happen in the loop
        order
            .match_stake_unmatched(stake_matched, liquidity.price)
            .map_err(|_| CoreError::MatchingPayoutAmountError)?;

        order_matches.push((liquidity.price, liquidity.sources.clone(), stake_matched));
    }

    // remove matched liquidity
    for (price, sources, stake) in &order_matches {
        market_liquidities
            .remove_liquidity_against(order.market_outcome_index, *price, sources, *stake)
            .map_err(|_| CoreError::MatchingRemainingLiquidityTooSmall)?;
        market_liquidities.update_stake_matched_total(*stake)?;

        for source in sources {
            let source_stake = calculate_stake_cross(*stake, *price, source.price);
            market_liquidities
                .remove_liquidity_for(source.outcome, source.price, &[], source_stake)
                .map_err(|_| CoreError::MatchingRemainingLiquidityTooSmall)?;
        }
    }

    // remainder is added to liquidities
    if order.stake_unmatched > 0_u64 {
        market_liquidities.add_liquidity_for(
            order.market_outcome_index,
            order.expected_price,
            order.stake_unmatched,
        )?;
    }

    Ok(order_matches
        .iter()
        .map(|(price, _, stake)| (*stake, *price))
        .collect())
}

fn match_against_order(
    market_liquidities: &mut MarketLiquidities,
    market_matching_queue: &mut MarketMatchingQueue,
    order_pk: &Pubkey,
    order: &mut Order,
) -> Result<Vec<(u64, f64)>> {
    let mut order_matches = Vec::with_capacity(MATCH_CAPACITY);
    let order_outcome = order.market_outcome_index;

    // AGAINST order matches FOR liquidity
    let liquidities = &market_liquidities.liquidities_for;

    for liquidity in liquidities
        .iter()
        .filter(|element| element.outcome == order_outcome)
    {
        if order.stake_unmatched == 0_u64 {
            break; // no need to loop any further
        }
        if order_matches.len() == order_matches.capacity() {
            break; // can't loop any further
        }
        if liquidity.price > order.expected_price {
            break; // liquidity.price <= expected_price must be true
        }

        let stake_matched = liquidity.liquidity.min(order.stake_unmatched);
        if liquidity.sources.is_empty() {
            // straight match
            market_matching_queue
                .matches
                .enqueue(OrderMatch::maker(
                    !order.for_outcome,
                    order.market_outcome_index,
                    liquidity.price,
                    stake_matched,
                ))
                .ok_or(MatchingQueueIsFull)?;
        } else {
            // cross match
            for liquidity_source in &liquidity.sources {
                let liquidity_source_stake_matched =
                    calculate_stake_cross(stake_matched, liquidity.price, liquidity_source.price);

                market_matching_queue
                    .matches
                    .enqueue(OrderMatch::maker(
                        order.for_outcome,
                        liquidity_source.outcome,
                        liquidity_source.price,
                        liquidity_source_stake_matched,
                    ))
                    .ok_or(MatchingQueueIsFull)?;
            }
        }

        // record taker match
        market_matching_queue
            .matches
            .enqueue(OrderMatch::taker(
                *order_pk,
                order.for_outcome,
                order.market_outcome_index,
                liquidity.price,
                stake_matched,
            ))
            .ok_or(MatchingQueueIsFull)?;

        // this needs to happen in the loop
        order
            .match_stake_unmatched(stake_matched, liquidity.price)
            .map_err(|_| CoreError::MatchingPayoutAmountError)?;

        order_matches.push((stake_matched, liquidity.price, liquidity.sources.clone()));
    }

    // remove matched liquidity
    for (stake, price, sources) in &order_matches {
        market_liquidities
            .remove_liquidity_for(order.market_outcome_index, *price, sources, *stake)
            .map_err(|_| CoreError::MatchingRemainingLiquidityTooSmall)?;
        market_liquidities.update_stake_matched_total(*stake)?;
    }

    // remainder is added to liquidities
    if order.stake_unmatched > 0_u64 {
        market_liquidities.add_liquidity_against(
            order.market_outcome_index,
            order.expected_price,
            order.stake_unmatched,
        )?;
    }

    Ok(order_matches
        .iter()
        .map(|(stake, price, _)| (*stake, *price))
        .collect())
}

#[cfg(test)]
mod test_match_for_order {
    use crate::instructions::matching::on_order_creation;
    use crate::instructions::matching::on_order_creation::{liquidities, liquidities2, matches};
    use crate::state::market_liquidities::{mock_market_liquidities, LiquidityKey};
    use crate::state::market_matching_queue_account::{MarketMatchingQueue, MatchingQueue};
    use crate::state::order_account::mock_order;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn straight_match() {
        let market_pk = Pubkey::new_unique();
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, 1, true, 2.8, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        market_liquidities
            .add_liquidity_against(1, 2.8, 125)
            .unwrap();
        market_liquidities
            .add_liquidity_against(2, 2.8, 125)
            .unwrap();
        market_liquidities
            .update_cross_liquidity_for(&[LiquidityKey::new(1, 2.8), LiquidityKey::new(2, 2.8)]);

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        )
        .expect("match_for_order");

        assert_eq!(
            vec!((3.5, 100)), // TODO incorrect - should be 20
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((2.8, 125), (2.8, 25)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec![(false, 2.8, 100), (true, 2.8, 100),],
            matches(&market_matching_queue.matches)
        );

        assert_eq!(0_u64, order.stake_unmatched);
        assert_eq!(280_u64, order.payout);
    }

    #[test]
    fn cross_match_3way() {
        let market_pk = Pubkey::new_unique();
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, 0, true, 3.5, 80, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        market_liquidities.add_liquidity_for(1, 2.8, 125).unwrap();
        market_liquidities.add_liquidity_for(2, 2.8, 125).unwrap();
        market_liquidities.update_cross_liquidity_against(&[
            LiquidityKey::new(1, 2.8),
            LiquidityKey::new(2, 2.8),
        ]);

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        )
        .expect("match_for_order");

        assert_eq!(
            vec![(2.8, 25), (2.8, 25)],
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec![(3.5, 3, 20)],
            liquidities2(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec![(true, 2.8, 100), (true, 2.8, 100), (true, 3.5, 80)],
            matches(&market_matching_queue.matches) // vec max length
        );

        assert_eq!(0_u64, order.stake_unmatched);
        assert_eq!(280_u64, order.payout);
    }

    #[test]
    fn cross_match_4way() {
        let market_pk = Pubkey::new_unique();
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, 0, true, 3.0, 120, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        market_liquidities.add_liquidity_for(1, 3.6, 200).unwrap();
        market_liquidities.add_liquidity_for(2, 4.0, 180).unwrap();
        market_liquidities.add_liquidity_for(3, 7.2, 100).unwrap();
        market_liquidities.update_cross_liquidity_against(&[
            LiquidityKey::new(1, 3.6),
            LiquidityKey::new(2, 4.0),
            LiquidityKey::new(3, 7.2),
        ]);

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        )
        .expect("match_for_order");

        assert_eq!(
            vec![(3.6, 100), (4.0, 90), (7.2, 50)],
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec![(3.0, 120)],
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec![
                (true, 3.6, 100),
                (true, 4.0, 90),
                (true, 7.2, 50),
                (true, 3.0, 120)
            ],
            matches(&market_matching_queue.matches)
        );

        assert_eq!(0_u64, order.stake_unmatched);
        assert_eq!(360_u64, order.payout);
    }
}

#[cfg(test)]
mod test_match_against_order {
    use crate::instructions::matching::on_order_creation;
    use crate::instructions::matching::on_order_creation::{liquidities, matches};
    use crate::state::market_liquidities::{mock_market_liquidities, LiquidityKey};
    use crate::state::market_matching_queue_account::{MarketMatchingQueue, MatchingQueue};
    use crate::state::order_account::mock_order;
    use solana_program::pubkey::Pubkey;

    #[test]
    fn straight_match() {
        let market_pk = Pubkey::new_unique();
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, 1, false, 2.8, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        market_liquidities.add_liquidity_for(1, 2.8, 125).unwrap();
        market_liquidities.add_liquidity_for(2, 2.8, 125).unwrap();
        market_liquidities.update_cross_liquidity_against(&[
            LiquidityKey::new(1, 2.8),
            LiquidityKey::new(2, 2.8),
        ]);

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        )
        .expect("");

        assert_eq!(
            vec!((2.8, 25), (2.8, 125)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((3.5, 100)), // TODO incorrect - should be 20
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec![(true, 2.8, 100), (false, 2.8, 100),],
            matches(&market_matching_queue.matches)
        );

        assert_eq!(0_u64, order.stake_unmatched);
        assert_eq!(280_u64, order.payout);
    }

    #[test]
    fn cross_match_3way() {
        let market_pk = Pubkey::new_unique();
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, 0, false, 3.5, 80, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        market_liquidities
            .add_liquidity_against(1, 2.8, 125)
            .unwrap();
        market_liquidities
            .add_liquidity_against(2, 2.8, 125)
            .unwrap();
        market_liquidities
            .update_cross_liquidity_for(&[LiquidityKey::new(1, 2.8), LiquidityKey::new(2, 2.8)]);

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        )
        .expect("");

        assert_eq!(
            vec!((3.5, 20)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec![(2.8, 125), (2.8, 125)], // TODO should be (2.8, 25), (2.8, 25)
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec![(false, 2.8, 100), (false, 2.8, 100), (false, 3.5, 80)],
            matches(&market_matching_queue.matches) // vec max length
        );

        assert_eq!(0_u64, order.stake_unmatched);
        assert_eq!(280_u64, order.payout);
    }
}

#[cfg(test)]
mod test {
    use crate::state::market_liquidities::mock_market_liquidities;
    use crate::state::market_matching_queue_account::MatchingQueue;

    use super::*;

    #[test]
    fn match_against_order_stop_after_fully_matched() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, false, 1.5, 10, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_for(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        )
        .expect("");

        assert_eq!(
            vec!((1.3, 10), (1.4, 10)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            Vec::<(f64, u64)>::new(),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec!((true, 1.2, 10), (false, 1.2, 10)),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(0_u64, order.stake_unmatched);
        assert_eq!(12_u64, order.payout);
    }

    #[test]
    fn match_against_order_with_more_matches_than_alloc() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![
            1.2, 1.25, 1.3, 1.35, 1.4, 1.45, 1.5, 1.55, 1.6, 1.65, 1.7, 1.75,
        ];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, false, 1.8, 120, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_for(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(30),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.7, 10), (1.75, 10)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.8, 20)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(100_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            vec![
                (true, 1.2, 10),
                (false, 1.2, 10),
                (true, 1.25, 10),
                (false, 1.25, 10),
                (true, 1.3, 10),
                (false, 1.3, 10),
                (true, 1.35, 10),
                (false, 1.35, 10),
                (true, 1.4, 10),
                (false, 1.4, 10),
                (true, 1.45, 10),
                (false, 1.45, 10),
                (true, 1.5, 10),
                (false, 1.5, 10),
                (true, 1.55, 10),
                (false, 1.55, 10),
                (true, 1.6, 10),
                (false, 1.6, 10),
                (true, 1.65, 10),
                (false, 1.65, 10)
            ],
            matches(&market_matching_queue.matches) // vec max length
        );

        assert_eq!(20_u64, order.stake_unmatched);
        assert_eq!(140_u64, order.payout);
    }

    #[test]
    fn match_against_order_with_price_1_1() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, false, 1.1, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_for(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.2, 10), (1.3, 10), (1.4, 10)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.1, 100)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(0_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            Vec::<(bool, f64, u64)>::new(),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(100_u64, order.stake_unmatched);
        assert_eq!(0_u64, order.payout);
    }

    #[test]
    fn match_against_order_with_price_1_2() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, false, 1.2, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_for(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.3, 10), (1.4, 10)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.2, 90)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec!((true, 1.2, 10), (false, 1.2, 10)),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(90_u64, order.stake_unmatched);
        assert_eq!(12_u64, order.payout);
    }

    #[test]
    fn match_against_order_with_price_1_3() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, false, 1.3, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_for(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.4, 10)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.3, 80)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(20_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            vec!(
                (true, 1.2, 10),
                (false, 1.2, 10),
                (true, 1.3, 10),
                (false, 1.3, 10)
            ),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(80_u64, order.stake_unmatched);
        assert_eq!(25_u64, order.payout);
    }

    #[test]
    fn match_against_order_with_price_1_4() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, false, 1.4, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_for(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            Vec::<(f64, u64)>::new(),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.4, 70)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(30_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            vec!(
                (true, 1.2, 10),
                (false, 1.2, 10),
                (true, 1.3, 10),
                (false, 1.3, 10),
                (true, 1.4, 10),
                (false, 1.4, 10)
            ),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(70_u64, order.stake_unmatched);
        assert_eq!(39_u64, order.payout);
    }

    #[test]
    fn match_against_order_with_price_1_5() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, false, 1.5, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_for(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            Vec::<(f64, u64)>::new(),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.5, 70)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(30_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            vec!(
                (true, 1.2, 10),
                (false, 1.2, 10),
                (true, 1.3, 10),
                (false, 1.3, 10),
                (true, 1.4, 10),
                (false, 1.4, 10)
            ),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(70_u64, order.stake_unmatched);
        assert_eq!(39_u64, order.payout);
    }

    #[test]
    fn match_for_order_stop_after_fully_matched() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, true, 1.1, 10, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_against(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            Vec::<(f64, u64)>::new(),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.3, 10), (1.2, 10)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec!((false, 1.4, 10), (true, 1.4, 10)),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(0_u64, order.stake_unmatched);
        assert_eq!(14_u64, order.payout);
    }

    #[test]
    fn match_for_order_with_more_matches_than_alloc() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4, 1.5, 1.6, 1.7];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, true, 1.1, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_against(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(30),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.1, 40)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            Vec::<(f64, u64)>::new(),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(60_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            vec!(
                (false, 1.7, 10),
                (true, 1.7, 10),
                (false, 1.6, 10),
                (true, 1.6, 10),
                (false, 1.5, 10),
                (true, 1.5, 10),
                (false, 1.4, 10),
                (true, 1.4, 10),
                (false, 1.3, 10),
                (true, 1.3, 10),
                (false, 1.2, 10),
                (true, 1.2, 10)
            ),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(40_u64, order.stake_unmatched);
        assert_eq!(87_u64, order.payout);
    }

    #[test]
    fn match_for_order_with_price_1_1() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, true, 1.1, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_against(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.1, 70)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            Vec::<(f64, u64)>::new(),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(30_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            vec!(
                (false, 1.4, 10),
                (true, 1.4, 10),
                (false, 1.3, 10),
                (true, 1.3, 10),
                (false, 1.2, 10),
                (true, 1.2, 10)
            ),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(70_u64, order.stake_unmatched);
        assert_eq!(39_u64, order.payout);
    }

    #[test]
    fn match_for_order_with_price_1_2() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, true, 1.2, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_against(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.2, 70)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            Vec::<(f64, u64)>::new(),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(30_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            vec!(
                (false, 1.4, 10),
                (true, 1.4, 10),
                (false, 1.3, 10),
                (true, 1.3, 10),
                (false, 1.2, 10),
                (true, 1.2, 10)
            ),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(70_u64, order.stake_unmatched);
        assert_eq!(39_u64, order.payout);
    }

    #[test]
    fn match_for_order_with_price_1_3() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, true, 1.3, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_against(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.3, 80)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.2, 10)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(20_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            vec!(
                (false, 1.4, 10),
                (true, 1.4, 10),
                (false, 1.3, 10),
                (true, 1.3, 10),
            ),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(80_u64, order.stake_unmatched);
        assert_eq!(27_u64, order.payout);
    }

    #[test]
    fn match_for_order_with_price_1_4() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, true, 1.4, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_against(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.4, 90)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.3, 10), (1.2, 10)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(
            vec!((false, 1.4, 10), (true, 1.4, 10)),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(90_u64, order.stake_unmatched);
        assert_eq!(14_u64, order.payout);
    }

    #[test]
    fn match_for_order_with_price_1_5() {
        let market_pk = Pubkey::new_unique();
        let market_outcome_index = 1;
        let market_price_ladder = vec![1.2, 1.3, 1.4];
        let payer_pk = Pubkey::new_unique();

        let order_pk = Pubkey::new_unique();
        let mut order = mock_order(market_pk, market_outcome_index, true, 1.5, 100, payer_pk);

        let mut market_liquidities = mock_market_liquidities(market_pk);
        for price in market_price_ladder.iter() {
            market_liquidities
                .add_liquidity_against(market_outcome_index, *price, 10)
                .unwrap();
        }

        let mut market_matching_queue = MarketMatchingQueue {
            market: market_pk,
            matches: MatchingQueue::new(10),
        };

        let on_order_creation_result = on_order_creation(
            &mut market_liquidities,
            &mut market_matching_queue,
            &order_pk,
            &mut order,
        );

        assert!(on_order_creation_result.is_ok());

        assert_eq!(
            vec!((1.5, 100)),
            liquidities(&market_liquidities.liquidities_for)
        );
        assert_eq!(
            vec!((1.4, 10), (1.3, 10), (1.2, 10)),
            liquidities(&market_liquidities.liquidities_against)
        );
        assert_eq!(0_u64, market_liquidities.stake_matched_total);
        assert_eq!(
            Vec::<(bool, f64, u64)>::new(),
            matches(&market_matching_queue.matches)
        );

        assert_eq!(100_u64, order.stake_unmatched);
        assert_eq!(0_u64, order.payout);
    }
}

#[cfg(test)]
fn liquidities(liquidities: &Vec<MarketOutcomePriceLiquidity>) -> Vec<(f64, u64)> {
    liquidities
        .iter()
        .map(|v| (v.price, v.liquidity))
        .collect::<Vec<(f64, u64)>>()
}

#[cfg(test)]
fn liquidities2(liquidities: &Vec<MarketOutcomePriceLiquidity>) -> Vec<(f64, u16, u64)> {
    liquidities
        .iter()
        .map(|v| {
            (
                v.price,
                MarketLiquidities::sources_ord(&v.sources),
                v.liquidity,
            )
        })
        .collect::<Vec<(f64, u16, u64)>>()
}

#[cfg(test)]
fn matches(matches: &MatchingQueue) -> Vec<(bool, f64, u64)> {
    matches
        .to_vec()
        .iter()
        .map(|v| (v.for_outcome, v.price, v.stake))
        .collect::<Vec<(bool, f64, u64)>>()
}
