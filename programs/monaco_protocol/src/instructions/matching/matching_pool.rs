use anchor_lang::prelude::*;

use crate::state::market_account::{Market, MarketStatus};
use crate::state::market_matching_pool_account::MarketMatchingPool;
use crate::state::market_matching_queue_account::MarketMatchingQueue;
use crate::{CoreError, Order};

pub fn update_on_match(
    market_matching_pool_against: &mut Account<MarketMatchingPool>,
    market_matching_pool_for: &mut Account<MarketMatchingPool>,
    for_order: &Account<Order>,
    against_order: &Account<Order>,
) -> Result<()> {
    let for_fully_matched = for_order.stake_unmatched == 0_u64;
    let against_fully_matched = against_order.stake_unmatched == 0_u64;

    // Update the pools
    update_matching_pool_with_matched_order(
        market_matching_pool_for,
        for_order.key(),
        for_fully_matched,
    )?;
    update_matching_pool_with_matched_order(
        market_matching_pool_against,
        against_order.key(),
        against_fully_matched,
    )?;

    Ok(())
}

pub fn update_matching_pool_with_new_order(
    market_matching_pool: &mut MarketMatchingPool,
    order_account: &Account<Order>,
) -> Result<()> {
    if order_account.stake_unmatched > 0 {
        market_matching_pool
            .orders
            .enqueue(order_account.key())
            .ok_or(CoreError::MatchingQueueIsFull)?;
    }

    Ok(())
}

pub fn move_market_matching_pool_to_inplay(
    market: &Market,
    market_matching_queue: &MarketMatchingQueue,
    market_matching_pool: &mut MarketMatchingPool,
) -> Result<()> {
    require!(
        market.market_status == MarketStatus::Open,
        CoreError::MatchingMarketInvalidStatus
    );
    require!(
        market.inplay_enabled,
        CoreError::MatchingMarketInplayNotEnabled
    );
    require!(market.is_inplay(), CoreError::MatchingMarketNotYetInplay);
    require!(
        !market_matching_pool.inplay,
        CoreError::MatchingMarketMatchingPoolAlreadyInplay
    );
    require!(
        market_matching_queue.matches.is_empty(),
        CoreError::InplayTransitionMarketMatchingQueueIsNotEmpty
    );
    market_matching_pool.move_to_inplay(&market.event_start_order_behaviour);
    Ok(())
}

pub fn update_matching_pool_with_matched_order(
    matching_pool: &mut MarketMatchingPool,
    matched_order: Pubkey,
    fully_matched: bool,
) -> Result<()> {
    let front_of_pool = match fully_matched {
        true => matching_pool.orders.dequeue(),
        false => matching_pool.orders.peek(0),
    };

    match front_of_pool {
        Some(pool_item) => {
            require!(
                &matched_order == pool_item,
                CoreError::OrderNotAtFrontOfQueue
            );
        }
        None => return Err(anchor_lang::error!(CoreError::MatchingQueueIsEmpty)),
    }

    Ok(())
}

pub fn update_on_cancel(
    market: &Market,
    market_matching_queue: &MarketMatchingQueue,
    matching_pool: &mut MarketMatchingPool,
    order: &Account<Order>,
) -> Result<bool> {
    if market.is_inplay() && !matching_pool.inplay {
        require!(
            market_matching_queue.matches.is_empty(),
            CoreError::InplayTransitionMarketMatchingQueueIsNotEmpty
        );
        matching_pool.move_to_inplay(&market.event_start_order_behaviour);
    }

    Ok(matching_pool.orders.remove(&order.key()).is_some())
}
