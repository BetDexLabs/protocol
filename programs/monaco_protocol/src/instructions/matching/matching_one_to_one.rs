use anchor_lang::prelude::*;

use crate::context::MatchOrders;
use crate::error::CoreError;
use crate::events::trade::TradeEvent;
use crate::instructions::market_position::update_product_commission_contributions;
use crate::instructions::matching::create_trade::initialize_trade;
use crate::instructions::{
    calculate_risk_from_stake, current_timestamp, matching, order, transfer,
};
use crate::state::market_account::MarketStatus::Open;
use crate::state::market_position_account::MarketPosition;

#[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
use crate::allocator::A;

pub fn match_orders(ctx: &mut Context<MatchOrders>) -> Result<()> {
    let order_for = &mut ctx.accounts.order_for;
    let order_against = &mut ctx.accounts.order_against;

    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    let mut before: usize;

    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        before = A.pos();
    }

    // validate market
    require!(
        Open.eq(&ctx.accounts.market.market_status),
        CoreError::MarketNotOpen,
    );

    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after validate market: {}", before - A.pos());
        before = A.pos();
    }

    let now = current_timestamp();
    require!(
        order_for.creation_timestamp <= ctx.accounts.market.market_lock_timestamp
            && order_against.creation_timestamp <= ctx.accounts.market.market_lock_timestamp,
        CoreError::MarketLocked
    );
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after validate market not locked: {}", before - A.pos());
        before = A.pos();
    }
    // validate orders market-outcome-price
    require!(
        order_for.market_outcome_index == order_against.market_outcome_index,
        CoreError::MatchingMarketOutcomeMismatch
    );
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after validate outcomes: {}", before - A.pos());
        before = A.pos();
    }
    require!(
        order_for.expected_price <= order_against.expected_price,
        CoreError::MatchingMarketPriceMismatch
    );
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after validate prices: {}", before - A.pos());
        before = A.pos();
    }
    // validate that status is open or matched (for partial matches)
    require!(!order_for.is_completed(), CoreError::StatusClosed);
    require!(!order_against.is_completed(), CoreError::StatusClosed);
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after validate order status: {}", before - A.pos());
        before = A.pos();
    }
    // validate that both orders are not within their inplay delay
    require!(
        order_for.delay_expiration_timestamp < now
            && order_against.delay_expiration_timestamp < now,
        CoreError::InplayDelay
    );
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after validate inplay delay: {}", before - A.pos());
        before = A.pos();
    }
    let selected_price = if order_for.creation_timestamp < order_against.creation_timestamp {
        order_for.expected_price
    } else {
        order_against.expected_price
    };
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after select price: {}", before - A.pos());
        before = A.pos();
    }
    // determine the matchable stake
    let stake_matched = order_for.stake_unmatched.min(order_against.stake_unmatched);

    let market_position_against = &mut ctx.accounts.market_position_against;
    let market_position_for = &mut ctx.accounts.market_position_for;
    // for orders from the same purchaser market-position passed is the same account
    let market_position_identical = market_position_against.key() == market_position_for.key();

    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after set up market positions: {}", before - A.pos());
        before = A.pos();
    }

    let change_in_exposure_refund_against;
    let change_in_exposure_refund_for;

    if order_against.creation_timestamp <= order_for.creation_timestamp {
        // 1. match against
        // -----------------------------
        change_in_exposure_refund_against = order::match_order(
            order_against,
            market_position_against,
            stake_matched,
            selected_price,
        )?;
        if market_position_identical {
            copy_market_position(market_position_against, market_position_for);
        }
        #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
        unsafe {
            msg!("after 1 a<f: {}", before - A.pos());
            before = A.pos();
        }
        // 2. match for
        // -----------------------------
        change_in_exposure_refund_for = order::match_order(
            order_for,
            market_position_for,
            stake_matched,
            selected_price,
        )?;
        if market_position_identical {
            copy_market_position(market_position_for, market_position_against);
        }
        #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
        unsafe {
            msg!("after 2 a<f: {}", before - A.pos());
            before = A.pos();
        }
    } else {
        // 1. match for
        // -----------------------------
        change_in_exposure_refund_for = order::match_order(
            order_for,
            market_position_for,
            stake_matched,
            selected_price,
        )?;
        if market_position_identical {
            copy_market_position(market_position_for, market_position_against);
        }
        #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
        unsafe {
            msg!("after 1 f<a: {}", before - A.pos());
            before = A.pos();
        }
        // 2. match against
        // -----------------------------
        change_in_exposure_refund_against = order::match_order(
            order_against,
            market_position_against,
            stake_matched,
            selected_price,
        )?;
        if market_position_identical {
            copy_market_position(market_position_against, market_position_for);
        }
        #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
        unsafe {
            msg!("after 2 f<a: {}", before - A.pos());
            before = A.pos();
        }
    };

    // update product commission tracking for matched risk
    update_product_commission_contributions(market_position_for, order_for, stake_matched)?;
    update_product_commission_contributions(
        market_position_against,
        order_against,
        calculate_risk_from_stake(stake_matched, selected_price),
    )?;
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after update product contribs: {}", before - A.pos());
        before = A.pos();
    }
    // 3. market update
    // -----------------------------
    matching::update_on_match(
        &mut ctx.accounts.market_outcome,
        &mut ctx.accounts.market_matching_pool_against,
        &mut ctx.accounts.market_matching_pool_for,
        &ctx.accounts.market.key(),
        stake_matched,
        order_for,
        order_against,
    )?;
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after 3 market update: {}", before - A.pos());
        before = A.pos();
    }

    // 4. if any refunds are due to change in exposure, transfer them
    if change_in_exposure_refund_against > 0_u64 {
        transfer::order_against_matching_refund(ctx, change_in_exposure_refund_against)?;
    }
    if change_in_exposure_refund_for > 0_u64 {
        transfer::order_for_matching_refund(ctx, change_in_exposure_refund_for)?;
    }
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after 4 matching refunds: {}", before - A.pos());
        before = A.pos();
    }

    // 5. Initialize the trade accounts
    let now = current_timestamp();
    initialize_trade(
        &mut ctx.accounts.trade_against,
        &ctx.accounts.order_against,
        &ctx.accounts.trade_for,
        stake_matched,
        selected_price,
        now,
        ctx.accounts.crank_operator.key(),
    );
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after trade init: {}", before - A.pos());
        before = A.pos();
    }
    ctx.accounts.market.increment_unclosed_accounts_count()?;
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after account # bump: {}", before - A.pos());
        before = A.pos();
    }
    initialize_trade(
        &mut ctx.accounts.trade_for,
        &ctx.accounts.order_for,
        &ctx.accounts.trade_against,
        stake_matched,
        selected_price,
        now,
        ctx.accounts.crank_operator.key(),
    );
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after trade init: {}", before - A.pos());
        before = A.pos();
    }
    ctx.accounts.market.increment_unclosed_accounts_count()?;
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after account # bump: {}", before - A.pos());
        before = A.pos();
    }
    emit!(TradeEvent {
        amount: stake_matched,
        price: selected_price,
        market: ctx.accounts.market.key(),
    });
    #[cfg(all(feature = "custom-heap", target_arch = "bpf"))]
    unsafe {
        msg!("after trade event emit: {}", before - A.pos());
    }

    Ok(())
}

fn copy_market_position(from: &MarketPosition, to: &mut MarketPosition) {
    for index in 0..from.market_outcome_sums.len() {
        to.market_outcome_sums[index] = from.market_outcome_sums[index];
        to.unmatched_exposures[index] = from.unmatched_exposures[index];
    }
}
