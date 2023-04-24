use {anchor_lang::prelude::*, perpetuals::state::position::Side};

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum Filled {
    Partial,
    Full,
}

impl Default for Filled {
    fn default() -> Self {
        Self::Partial
    }
}

#[derive(Copy, Clone, PartialEq, AnchorSerialize, AnchorDeserialize, Debug)]
pub enum OrderType {
    StopLimit,
    StopMarket,
    TakeProfit,
    TakeProfitLimit,
    Market,
    Limit,
}

impl Default for OrderType {
    fn default() -> Self {
        Self::StopLimit
    }
}

#[account]
#[derive(Default, Debug)]
pub struct Order {
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub side: Side,
    pub price: u64,
    pub trigger_price: u64,
    pub limit_price: u64,

    pub funding_account: Pubkey,
    pub receiving_custody: Pubkey,
    pub dispensing_custody: Pubkey,
    pub collateral: u64,
    pub size: u64,
    pub order_time: i64,
    pub expiry_time: i64,
    pub order_type: OrderType,

    pub bump: u8,
}

impl Order {
    pub const LEN: usize = 8 + std::mem::size_of::<Order>();
}
