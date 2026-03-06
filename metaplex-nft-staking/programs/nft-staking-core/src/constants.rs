use anchor_lang::prelude::*;
pub const SECONDS_IN_A_DAY: i64 = 86400;
pub const NINE_AM_UTC: i64 = 9 * 3600; // 32,400
pub const FIVE_PM_UTC: i64 = 17 * 3600; // 61,200
pub const MARGIN: i64 = 3600;
pub const REWARD_IN_LAMPORTS: u64 = 1_000_000;
#[constant]
pub const ORACLE_ACCOUNT: Pubkey =
    Pubkey::from_str_const("53DY5i9HL2bYoB8nD2yjVoaNZ8RLp3Lmr99VfFy4U8eF");
