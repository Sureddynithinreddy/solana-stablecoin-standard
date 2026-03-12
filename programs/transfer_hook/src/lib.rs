use anchor_lang::prelude::*;

declare_id!("C7TEPRE4dATYHHV3wafWybBD6z5FkngfAuBVNvezmBA4");

pub mod stablecoin_program {
    use anchor_lang::declare_id;
    declare_id!("CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh");
}

#[program]
pub mod transfer_hook {
    use super::*;

    pub fn validate_transfer(ctx: Context<ValidateTransfer>) -> Result<()> {
        // If blacklist PDA exists -> fail
        if !ctx.accounts.sender_blacklist.data_is_empty() {
             return err!(ErrorCode::Blacklisted);
        }

        if !ctx.accounts.receiver_blacklist.data_is_empty() {
             return err!(ErrorCode::Blacklisted);
        }

        msg!("Transfer allowed");

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ValidateTransfer<'info> {
    /// CHECK: sender
    pub sender: AccountInfo<'info>,

    /// CHECK: receiver
    pub receiver: AccountInfo<'info>,

    #[account(
        seeds = [b"blacklist", sender.key().as_ref()],
        bump,
        seeds::program = stablecoin_program::ID
    )]
    /// CHECK: sender blacklist PDA
    pub sender_blacklist: AccountInfo<'info>,

    #[account(
        seeds = [b"blacklist", receiver.key().as_ref()],
        bump,
        seeds::program = stablecoin_program::ID
    )]
    /// CHECK: receiver blacklist PDA
    pub receiver_blacklist: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Wallet is blacklisted")]
    Blacklisted,
}