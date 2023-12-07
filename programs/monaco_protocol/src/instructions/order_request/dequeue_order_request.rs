use crate::error::CoreError;
use crate::instructions::market_position;
use crate::state::market_order_request_queue::MarketOrderRequestQueue;
use crate::state::market_position_account::MarketPosition;
use anchor_lang::prelude::*;

pub fn dequeue_order_request(
    order_request_queue: &mut MarketOrderRequestQueue,
    market_position: &mut MarketPosition,
) -> Result<u64> {
    let order_request = order_request_queue
        .order_requests
        .dequeue()
        .ok_or(CoreError::OrderRequestQueueIsEmpty)?;

    require!(
        order_request.purchaser == market_position.purchaser,
        CoreError::CancelationPurchaserMismatch
    );

    // calculate refund
    market_position::update_on_order_request_cancellation(market_position, order_request)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::market_order_request_queue::{OrderRequest, OrderRequestQueue};

    #[test]
    fn dequeue_order_request_ok() {
        let purchaser = Pubkey::new_unique();
        let order_request = OrderRequest {
            purchaser,
            market_outcome_index: 0,
            for_outcome: true,
            product: None,
            stake: 10,
            expected_price: 3.0,
            delay_expiration_timestamp: 0,
            product_commission_rate: 0.0,
            distinct_seed: [0; 16],
            creation_timestamp: 0,
        };
        let expected_refund = order_request.stake;

        let order_request_queue = &mut MarketOrderRequestQueue {
            market: Pubkey::new_unique(),
            order_requests: OrderRequestQueue::new(10),
        };
        order_request_queue.order_requests.enqueue(order_request);

        let market_position = &mut MarketPosition {
            purchaser,
            market: Default::default(),
            paid: false,
            market_outcome_sums: vec![10, -10, -10],
            unmatched_exposures: vec![0, 10, 10],
            payer: Default::default(),
            matched_risk: 0,
            matched_risk_per_product: vec![],
        };

        let result = dequeue_order_request(order_request_queue, market_position);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_refund)
    }

    #[test]
    fn dequeue_order_request_purchaser_mismatch() {
        let order_request = OrderRequest {
            purchaser: Pubkey::new_unique(),
            market_outcome_index: 0,
            for_outcome: true,
            product: None,
            stake: 10,
            expected_price: 3.0,
            delay_expiration_timestamp: 0,
            product_commission_rate: 0.0,
            distinct_seed: [0; 16],
            creation_timestamp: 0,
        };

        let order_request_queue = &mut MarketOrderRequestQueue {
            market: Pubkey::new_unique(),
            order_requests: OrderRequestQueue::new(10),
        };
        order_request_queue.order_requests.enqueue(order_request);

        let market_position = &mut MarketPosition {
            purchaser: Pubkey::new_unique(),
            market: Default::default(),
            paid: false,
            market_outcome_sums: vec![10, -10, -10],
            unmatched_exposures: vec![0, 10, 10],
            payer: Default::default(),
            matched_risk: 0,
            matched_risk_per_product: vec![],
        };

        let result = dequeue_order_request(order_request_queue, market_position);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            error!(CoreError::CancelationPurchaserMismatch)
        );
    }

    #[test]
    fn dequeue_order_request_queue_empty() {
        let order_request_queue = &mut MarketOrderRequestQueue {
            market: Pubkey::new_unique(),
            order_requests: OrderRequestQueue::new(10),
        };

        let market_position = &mut MarketPosition {
            purchaser: Default::default(),
            market: Default::default(),
            paid: false,
            market_outcome_sums: vec![10, -10, -10],
            unmatched_exposures: vec![0, 10, 10],
            payer: Default::default(),
            matched_risk: 0,
            matched_risk_per_product: vec![],
        };

        let result = dequeue_order_request(order_request_queue, market_position);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            error!(CoreError::OrderRequestQueueIsEmpty)
        );
    }
}
