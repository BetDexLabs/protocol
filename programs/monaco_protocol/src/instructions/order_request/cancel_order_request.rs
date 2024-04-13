use crate::error::CoreError;
use crate::instructions::market_position;
use anchor_lang::prelude::*;

use crate::state::market_account::{Market, MarketStatus};
use crate::state::market_order_request_queue::MarketOrderRequestQueue;
use crate::state::market_position_account::MarketPosition;

pub fn cancel_order_request(
    market: &mut Market,
    market_position: &mut MarketPosition,
    order_request_queue: &mut MarketOrderRequestQueue,
    distinct_seed: [u8; 16],
) -> Result<u64> {
    require!(
        [MarketStatus::Open].contains(&market.market_status),
        CoreError::CancelOrderNotCancellable
    );

    let order_request = order_request_queue
        .remove_order_request(distinct_seed)
        .ok_or(CoreError::CancelOrderNotCancellable)?;

    // calculate refund
    market_position::update_on_order_request_cancellation(market_position, &order_request)
}
