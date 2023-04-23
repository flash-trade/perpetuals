//! Error types

use anchor_lang::prelude::*;

#[error_code]
pub enum LimitOrderError {
    #[msg("Invalid authority")]
    InvalidAuthority,
    #[msg("Overflow in arithmetic operation")]
    MathOverflow,
    #[msg("Instruction is not allowed in production")]
    InvalidEnvironment,
    #[msg("Insufficient token amount returned")]
    InsufficientAmountReturned,
    #[msg("Token is not supported")]
    UnsupportedToken,
    #[msg("Instruction is not allowed at this time")]
    InstructionNotAllowed,
    #[msg("The provided program data is incorrect.")]
    InvalidProgramData,
    #[msg("The provided program upgrade authority is incorrect.")]
    InvalidProgramUpgradeAuthority,
    #[msg("The order is expired.")]
    OrderExpired,
    #[msg("The order is not expired.")]
    OrderNotExpired,
    #[msg("Price slippage limit exceeded")]
    MaxPriceSlippage,
}
