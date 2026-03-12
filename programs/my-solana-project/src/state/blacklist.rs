use anchor_lang::prelude::*;

#[account]
pub struct Blacklist {
    pub wallet: Pubkey,
}