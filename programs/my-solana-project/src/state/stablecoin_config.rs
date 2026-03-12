use anchor_lang::prelude::*;

#[account]
pub struct StablecoinConfig{
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub treasury: Pubkey,
    pub decimals: u8,
     pub paused: bool,
     pub minter: Pubkey
}