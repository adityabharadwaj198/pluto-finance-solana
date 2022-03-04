use anchor_lang::prelude::*;

declare_id!("5teAQ8gkGsfCAVpuHUWqtZsqJEUj52xn4nDjhTDDSQk1");
//declare_id!("5YbxmN9rS7RVuMEH458fpW8cN39vGmhuMkiu2EHa3J6x");

#[program]
pub mod oracle {
    use super::*;
    pub fn initialize_oracle(ctx: Context<Initialize>, admin:Pubkey, initial_price: u64, ticker: Vec<u8>) -> ProgramResult {
        let equity = &mut ctx.accounts.price_feed;
        // equity.bump = bump;
        equity.symbol = ticker;
        equity.admin = admin;
        equity.price = initial_price;
        equity.paused = false;
        Ok(())
    }

    pub fn set_paused(ctx: Context<Pause>, paused: bool) -> ProgramResult {
        let equity = &mut ctx.accounts.price_feed;
        equity.paused = paused;
        Ok(())
    }

    pub fn set_price(ctx: Context<SetPrice>, price: u64) -> ProgramResult {
        let equity = &mut ctx.accounts.price_feed;
        equity.price = price;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer=admin, space=9000)]
    pub price_feed: Account<'info, PriceFeed>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(mut, has_one = admin)]
    pub price_feed: Account<'info, PriceFeed>,
    #[account(signer)]
    pub admin: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct SetPrice<'info> {
    #[account(mut, has_one = admin)]
    pub price_feed: Account<'info, PriceFeed>,
    #[account(signer)]
    pub admin: AccountInfo<'info>,
}

#[account]
pub struct PriceFeed {
    pub admin: Pubkey,
    pub price: u64,
    pub paused: bool,
    pub symbol: Vec<u8>,
    // pub bump: u8,
}
