use anchor_lang::prelude::*;

declare_id!("C7TEPRE4dATYHHV3wafWybBD6z5FkngfAuBVNvezmBA4");

#[program]
pub mod transfer_hook {
    use super::*;

    pub fn validate_transfer(ctx: Context<ValidateTransfer>) -> Result<()> {

        // If blacklist PDA exists -> lamports > 0
        if ctx.accounts.sender_blacklist.lamports() > 0 {
            return err!(ErrorCode::Blacklisted);
        }

        if ctx.accounts.receiver_blacklist.lamports() > 0 {
            return err!(ErrorCode::Blacklisted);
        }

        msg!("Transfer allowed");

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ValidateTransfer<'info> {

    /// CHECK
    pub sender: AccountInfo<'info>,

    /// CHECK
    pub receiver: AccountInfo<'info>,

    /// CHECK
    pub sender_blacklist: AccountInfo<'info>,

    /// CHECK
    pub receiver_blacklist: AccountInfo<'info>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Wallet is blacklisted")]
    Blacklisted,
}