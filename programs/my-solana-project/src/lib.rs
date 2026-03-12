use anchor_lang::prelude::*;
use anchor_spl::token::{
    self,
    Mint,
    Token,
    TokenAccount,
    MintTo,
    Burn,
    FreezeAccount,
    ThawAccount,
    Transfer,
};

declare_id!("CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh");

pub mod state;

use crate::state::stablecoin_config::StablecoinConfig;
#[account]
pub struct Blacklist {
    pub wallet: Pubkey,
}

#[program]
pub mod my_solana_project {
    use super::*;

    pub fn initialize_stablecoin(
        ctx: Context<InitializeStablecoin>,
        decimals: u8,
    ) -> Result<()> {

        let config = &mut ctx.accounts.config;

        config.admin = ctx.accounts.admin.key();
        config.mint = ctx.accounts.mint.key();
        config.treasury = ctx.accounts.treasury.key();
        config.decimals = decimals;
        config.paused = false;
        config.minter = ctx.accounts.admin.key();

        msg!("Stablecoin initialized");

        Ok(())
    }

    pub fn create_mint(
        _ctx: Context<CreateMint>,
        _decimals: u8,
    ) -> Result<()> {

        msg!("Stablecoin mint created");

        Ok(())
    }
    pub fn pause(ctx: Context<PauseProtocol>) -> Result<()> {

    let config = &mut ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );

    config.paused = true;

    msg!("Protocol paused");

    Ok(())
}
pub fn unpause(ctx: Context<PauseProtocol>) -> Result<()> {

    let config = &mut ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );

    config.paused = false;

    msg!("Protocol unpaused");

    Ok(())
}
    pub fn freeze_account(ctx: Context<FreezeUserAccount>) -> Result<()> {

    let config = &ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );
    require!(
    ctx.accounts.mint.key() == config.mint,
    ErrorCode::InvalidMint
);

    let cpi_accounts = FreezeAccount {
        account: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.admin.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::freeze_account(cpi_ctx)?;

    msg!("Account frozen successfully");

    Ok(())
}

pub fn thaw_account(ctx: Context<ThawUserAccount>) -> Result<()> {

    let config = &ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );
    require!(
    ctx.accounts.mint.key() == config.mint,
    ErrorCode::InvalidMint
);

    let cpi_accounts = ThawAccount {
        account: ctx.accounts.user_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.admin.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::thaw_account(cpi_ctx)?;

    msg!("Account unfrozen successfully");

    Ok(())
}

   pub fn mint_tokens(
    ctx: Context<MintTokens>,
    amount: u64,
) -> Result<()> {

    let config = &ctx.accounts.config;

    require!(!config.paused, ErrorCode::ProtocolPaused);

    require!(
        ctx.accounts.admin.key() == config.minter,
        ErrorCode::Unauthorized
    );

    require!(
        ctx.accounts.mint.key() == config.mint,
        ErrorCode::InvalidMint
    );

    let cpi_accounts = MintTo {
        mint: ctx.accounts.mint.to_account_info(),
        to: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.admin.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::mint_to(cpi_ctx, amount)?;

    msg!("Tokens minted successfully");

    Ok(())
}
    pub fn burn_tokens(
    ctx: Context<BurnTokens>,
    amount: u64,
) -> Result<()> {

    let config = &ctx.accounts.config;

    require!(!config.paused, ErrorCode::ProtocolPaused);
    require!(
    ctx.accounts.mint.key() == config.mint,
    ErrorCode::InvalidMint
);

    let cpi_accounts = Burn {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.user_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::burn(cpi_ctx, amount)?;

    msg!("Tokens burned successfully");

    Ok(())
}
pub fn update_minter(
    ctx: Context<UpdateMinter>,
    new_minter: Pubkey,
    ) -> Result<()> {

    let config = &mut ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );

    config.minter = new_minter;

    msg!("Minter updated");

    Ok(())
}
pub fn transfer_authority(
    ctx: Context<TransferAuthority>,
    new_admin: Pubkey,
) -> Result<()> {

    let config = &mut ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );

    config.admin = new_admin;

    msg!("Admin authority transferred");

    Ok(())
}
pub fn add_to_blacklist(
    ctx: Context<AddToBlacklist>,
    wallet: Pubkey,
) -> Result<()> {

    let config = &ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );

    let blacklist = &mut ctx.accounts.blacklist;

    blacklist.wallet = wallet;

    msg!("Wallet added to blacklist");

    Ok(())
}
pub fn remove_from_blacklist(
    ctx: Context<RemoveFromBlacklist>,
) -> Result<()> {

    let config = &ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );

    msg!("Wallet removed from blacklist");

    Ok(())
}
pub fn seize(
    ctx: Context<SeizeTokens>,
    amount: u64,
) -> Result<()> {

    let config = &ctx.accounts.config;

    require!(
        ctx.accounts.admin.key() == config.admin,
        ErrorCode::Unauthorized
    );

    require!(
        ctx.accounts.mint.key() == config.mint,
        ErrorCode::InvalidMint
    );

    let cpi_accounts = anchor_spl::token::Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.treasury_token_account.to_account_info(),
        authority: ctx.accounts.admin.to_account_info(),
    };

    let cpi_program = ctx.accounts.token_program.to_account_info();

    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    token::transfer(cpi_ctx, amount)?;

    msg!("Tokens seized to treasury");

    Ok(())
}
}

#[derive(Accounts)]
pub struct InitializeStablecoin<'info> {

    #[account(init, payer = admin, space = 8 + 32 + 32 + 32 + 1 + 1 + 32)]
    pub config: Account<'info, StablecoinConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: mint account
    pub mint: AccountInfo<'info>,

    /// CHECK: treasury account
    pub treasury: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateMint<'info> {

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Mint account
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Mint account
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: user token account
#[account(mut)]
pub user_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct BurnTokens<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    #[account(mut)]
    pub user: Signer<'info>,

    /// CHECK: Mint account
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: User token account
    /// CHECK: user token account
#[account(mut)]
pub user_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct FreezeUserAccount<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ThawUserAccount<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Mint account
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: User token account
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}
#[derive(Accounts)]
pub struct PauseProtocol<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    pub admin: Signer<'info>,
}
#[derive(Accounts)]
pub struct UpdateMinter<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    pub admin: Signer<'info>,
}
#[derive(Accounts)]
pub struct TransferAuthority<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    pub admin: Signer<'info>,
}
#[derive(Accounts)]
#[instruction(wallet: Pubkey)]
pub struct AddToBlacklist<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    #[account(
        init,
        payer = admin,
        space = 8 + 32,
        seeds = [b"blacklist", wallet.as_ref()],
        bump
    )]
    pub blacklist: Account<'info, Blacklist>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct RemoveFromBlacklist<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    #[account(
        mut,
        close = admin,
        seeds = [b"blacklist", blacklist.wallet.as_ref()],
        bump
    )]
    pub blacklist: Account<'info, Blacklist>,

    #[account(mut)]
    pub admin: Signer<'info>,
}
#[derive(Accounts)]
pub struct SeizeTokens<'info> {

    #[account(mut)]
    pub config: Account<'info, StablecoinConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    /// CHECK: Mint account
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// CHECK: User token account
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,

    /// CHECK: Treasury token account
    #[account(mut)]
    pub treasury_token_account: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Protocol is paused")]
    ProtocolPaused,
    
     #[msg("Invalid mint account")]
    InvalidMint,

    #[msg("Wallet is blacklisted")]
    Blacklisted,
}