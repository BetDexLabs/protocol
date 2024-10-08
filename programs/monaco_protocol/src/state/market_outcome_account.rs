use crate::state::type_size::*;
use anchor_lang::prelude::*;
use std::string::ToString;

#[account]
pub struct MarketOutcome {
    pub market: Pubkey,
    pub index: u16,
    pub title: String,
    pub prices: Option<Pubkey>,
    pub price_ladder: Vec<f64>,
}

impl MarketOutcome {
    pub const TITLE_MAX_LENGTH: usize = 100;
    pub const PRICE_LADDER_LENGTH: usize = 320;

    pub const SIZE: usize = DISCRIMINATOR_SIZE
        + PUB_KEY_SIZE // market
        + U16_SIZE // index
        + vec_size(CHAR_SIZE, MarketOutcome::TITLE_MAX_LENGTH) // title
        + option_size(PUB_KEY_SIZE) // price ladder account
        + vec_size(F64_SIZE, MarketOutcome::PRICE_LADDER_LENGTH); // price_ladder
}

#[cfg(test)]
pub fn mock_market_outcome(market_pk: Pubkey, outcome: u16) -> MarketOutcome {
    MarketOutcome {
        market: market_pk,
        index: outcome,
        title: market_pk.to_string(),
        prices: None,
        price_ladder: vec![],
    }
}
