use anchor_lang::prelude::*;

#[account]
pub struct StablecoinConfig {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub treasury: Pubkey,
    pub decimals: u8,
    pub paused: bool,
    pub minter: Pubkey,
    // Metadata
    pub name: String,
    pub symbol: String,
    pub uri: String,
    // SSS-2 compliance flags
    pub enable_permanent_delegate: bool,
    pub enable_transfer_hook: bool,
    pub default_account_frozen: bool,
}

impl StablecoinConfig {
    pub const INIT_SPACE: usize = 8 + 32 + 32 + 32 + 1 + 1 + 32 + (4 + 32) + (4 + 10) + (4 + 200) + 1 + 1 + 1;
}