use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, MintTo, TokenAccount, Transfer};
use pyth_client::{PriceType, PriceStatus, CorpAction, load_mapping, load_product, load_price};
mod math;
use math::*;


declare_id!("DgPzqjoWLNQWfLLFZBRWt29m65wFJdqjdxS7GKqMCbgU");
//declare_id!("5YbxmN9rS7RVuMEH458fpW8cN39vGmhuMkiu2EHa3J6x");

#[program] //each inner method defines an instruction (RPC request handler). These handlers are entrypoints to the program that the clients/other programs can invoke
pub mod club90 {
    use super::*;

    pub const ASSETS_SIZE: usize = 10;
    // pub const AAPL_PROD: str = "3mkwqdkawySvAm1VjD4f2THN5mmXzb76fvft2hWpAANo";
    // pub const APPL_PRICE: str = "5yixRcKtcs5BZ1K2FsLFwmES1MyA92d6efvijjVevQCw";

    pub fn start_stuff_off(ctx: Context<StartStuffOff>) -> ProgramResult {
        let base_account = &mut ctx.accounts.base_account;
        base_account.nonce = 0;
        base_account.signer = Pubkey::default();
        base_account.admin = Pubkey::default();
        base_account.mint_authority = Pubkey::default();
        base_account.initialized = false;
        base_account.debt = 0;
        base_account.shares = 0;
        base_account.collateral_balance = 0;
        base_account.collateralization_level = 500;
        base_account.max_delay = 1000;
        base_account.fee = 30;
        base_account.collateral_token = Pubkey::default();
        base_account.collateral_account = Pubkey::default();
        let mut assets: Vec<Asset> = vec![];
        assets.resize( //asset vector will be resized with 10 as the size & every asset having ticker from 1 to 10.
            ASSETS_SIZE,
            Asset {
                ticker: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10], //why is  ticker made an array
                ..Default::default()
            }, 
        );
        base_account.assets = assets;
        Ok(())
    }
    
    pub fn initialize(ctx: Context<Initialize>, nonce: u8,
            signer: Pubkey, admin: Pubkey,
            collateral_token: Pubkey, collateral_account: Pubkey,
            collateral_token_feed: Pubkey, usd_token: Pubkey, //these aren't variables inside the baseaccount.
            mint_authority: Pubkey) -> ProgramResult {
                let base_account = &mut ctx.accounts.base_account;
                base_account.initialized = true;
                base_account.signer = signer;
                base_account.nonce = nonce;
                base_account.admin = admin;
                base_account.collateral_token = collateral_token;
                base_account.collateral_account = collateral_account;
                base_account.mint_authority = mint_authority;
                let usd_asset = Asset {
                    decimals: 8,
                    asset_address: usd_token,
                    feed_address: Pubkey::default(), // unused -> meaning for usd token we don't need a feed address 
                    last_update: std::u64::MAX, // find meaning of this 
                    price: 1 * 10u64.pow(4), 
                    supply: 0,
                    ticker: "xUSD".as_bytes().to_vec(),
                };
                let collateral_asset = Asset {
                    decimals: 8,
                    asset_address: collateral_token, //coming from arguments 
                    feed_address: collateral_token_feed, //coming from arguments
                    last_update: 0, 
                    price: 0,
                    supply: 0,
                    ticker: "CLUB".as_bytes().to_vec(),
                };
                base_account.assets = vec![usd_asset, collateral_asset];
                Ok(())
        }

    pub fn create_user_account(ctx: Context<CreateUserAccount>, owner: Pubkey) -> ProgramResult {
        let user_account = &mut ctx.accounts.user_account;
        user_account.owner = owner;
        user_account.shares = 0;
        user_account.collateral = 0;
        Ok(())
    }

    pub fn mint(ctx: Context<Mint>, amount: u64) -> ProgramResult {
        {
        let base_account = &mut ctx.accounts.base_account; //taking base account out of the context 
        let user_account = &mut ctx.accounts.user_account; //taking user account out of the context 
        let mint_token_address = ctx.accounts.mint.to_account_info().clone().key; 
        if !mint_token_address.eq(&base_account.assets[0].asset_address) {
            return Err(ErrorCode::NotSyntheticUsd.into());
        }
        let slot = ctx.accounts.clock.slot;
        let debt = calculate_debt(&base_account.assets, slot, base_account.max_delay).unwrap(); //if this is the 1st user to interact with the platform debt will be 0 
        let user_debt = calculate_user_debt_in_usd(user_account, debt, base_account.shares); //calculating user's debt here -> which is the sum of USD price of all the assets he already owns  
        let collateral_asset = base_account //this is trying to get the collateral asset which the user will deposit to gain the synthetic token
                                .assets
                                .clone()
                                .into_iter()
                                .find(|x| x.asset_address == base_account.collateral_token)
                                .unwrap();
                                msg!("debt {}", debt);
        
        let max_user_debt = calculate_max_user_debt_in_usd( //this will get the maximum debt a user is allowed to borrow from the whole system
                                &collateral_asset,
                                base_account.collateralization_level,
                                user_account,
                                );
                    
        let mint_asset = &mut base_account //this will return the synthetic asset that the user is trying to mint (for example pTesla/pAAPL)
                            .assets
                            .clone()
                            .into_iter() //using iter_mut mutable `Vector`s can also be iterated over in a way that allows modifying each value
                            .find(|x| x.asset_address == *mint_token_address)
                            .unwrap();

        let amount_mint_usd = calculate_amount_mint_in_usd(&mint_asset, amount); //the asset to be minted (gotten from above) * price of that asset is what this function returns
        
        msg!("mint {}", 123);

        if max_user_debt - user_debt < amount_mint_usd { //adding a check here to make sure that user does not borrow more than his maximum debt
            return Err(ErrorCode::MintLimit.into());
        }
        let new_shares = calculate_new_shares(&base_account.shares, &debt, &amount_mint_usd); //if this is the first user new_shares will be 10^8.

        msg!("mint {}", 1234);
        base_account.debt = debt + amount_mint_usd; //this is the system's debt -> if this is the first user then it will be 0 + whatever amount was calculated in line#121
        base_account.shares += new_shares; //if first user -> this will become 0 + 10^8 (if second user -> this will become 10^8 + new_shares calculated from L#128) 
        user_account.shares += new_shares; //whatever new shares calculated in l#128 will be added to the user's account. 
        mint_asset.supply += amount; 
        }
        // let cpi_ctx = CpiContext::from(&*ctx.accounts).with_signer(signer); //Context specifying non-argument inputs for cross-program-invocations.
        
        let base_account = ctx.accounts.base_account.clone();
        let seeds = &[base_account.signer.as_ref(), &[base_account.nonce]];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::from(&*ctx.accounts).with_signer(signer);
        token::mint_to(cpi_ctx, amount).map(|err| println!("{:?}", err)).ok();
        Ok(())
    }

    pub fn add_asset(ctx: Context<AddAsset>, ticker: Vec<u8>) -> Result<()> {
        let base_account = &mut ctx.accounts.base_account;
        if !base_account.admin.eq(ctx.accounts.admin.key) { //if admin is not calling this function then throw unauthorized
            return Err(ErrorCode::Unauthorized.into());
        }
        if base_account.assets.len() == ASSETS_SIZE { //if assets' array size reaches 10 then throw assets full error 
            return Err(ErrorCode::AssetsFull.into());
        }

        for asset in base_account.assets.clone() { //Checking if asset already exists
            if asset.ticker == ticker {
                return Err(ErrorCode::AssetAlreadyExists.into());
            }
        }

        let new_asset = Asset {
            // prod_address: *ctx.accounts.prod_address.to_account_info().key, //asset address milega in ctx arguments 
            // price_address: *ctx.accounts.price_address.to_account_info().key, //feed address for oracle milega in context args
            asset_address: *ctx.accounts.asset_address.to_account_info().key,
            feed_address: *ctx.accounts.feed_address.to_account_info().key,
            price: 0, //price is set to 0
            supply: 0, // supply set to 0
            last_update: 0,
            decimals: 8, 
            ticker: ticker,
        };
        base_account.assets.push(new_asset);
        Ok(())
    }

    pub fn deposit(ctx:Context<Deposit>) -> Result<()> {
        let base_account = &mut ctx.accounts.base_account;
        let new_balance = ctx.accounts.collateral_account.amount;
        let deposited = new_balance - base_account.collateral_balance;
        if deposited == 0 {
            return Err(ErrorCode::ZeroDeposit.into());
        }
        let user_account = &mut ctx.accounts.user_account;
        user_account.collateral += deposited; //user ka collateral badh raha hai -> means he has deposited SNY 
        base_account.collateral_balance = new_balance; //after this the collateral balance will be set to the existing amt inside system's collateral acc. 
        Ok(())
    }        

    pub fn withdraw(ctx:Context<Withdraw>, amount: u64) -> Result<()> {
        {
        let base_account = &mut ctx.accounts.base_account;
        let user_account = &mut ctx.accounts.user_account;
        let slot = ctx.accounts.clock.slot;
        let debt = calculate_debt(&base_account.assets, slot, base_account.max_delay).unwrap(); //total debt of the system
        let user_debt = calculate_user_debt_in_usd(user_account, debt, base_account.shares); //total debt of the user (calculated in that wierd logic)

        let collateral_asset = base_account //find the asset that's kept as collateral 
                                .assets
                                .clone()
                                .into_iter()
                                .find(|x| x.asset_address == base_account.collateral_token)
                                .unwrap();

        let max_user_debt = calculate_max_user_debt_in_usd( //calculate max debt that a user can take on using the above collateral asset 
            &collateral_asset,
            base_account.collateralization_level,
            user_account,
        );
        let max_withdraw_in_usd = calculate_max_withdraw_in_usd( 
            &max_user_debt,
            &user_debt,
            &base_account.collateralization_level,
        );
        let max_amount_to_withdraw = max_withdraw_in_usd*10u64.pow(4)/collateral_asset.price; // oracle got 4 places offset                    
        msg!("max amount to withdraw : {:?}", max_amount_to_withdraw);
        if max_amount_to_withdraw < amount {
            return Err(ErrorCode::WithdrawError.into());
        }
        user_account.collateral -= amount;
        base_account.collateral_balance -= amount;
        }
        let base_account = ctx.accounts.base_account.clone();
        let seeds = &[base_account.signer.as_ref(), &[base_account.nonce]];
        let signer = &[&seeds[..]];
        let cpi_ctx = CpiContext::from(&*ctx.accounts).with_signer(signer);
        token::transfer(cpi_ctx, amount);
        Ok(())
    }

    pub fn burn(ctx: Context<BurnToken>, amount: u64) -> Result<()> {
        let base_account = ctx.accounts.base_account.clone();
        let user_account = &mut ctx.accounts.user_account;
        let token_address = ctx.accounts.mint.key;
        let slot = ctx.accounts.clock.slot;
        let debt = calculate_debt(&base_account.assets, slot, base_account.max_delay).unwrap(); //calculate system debt. (for all assets)
        msg!("Debt calculated inside burn {}", debt);
        let user_debt = calculate_user_debt_in_usd(user_account, debt, base_account.shares); //calculate debt of this particular user
        msg!("User debt inside burn {}", user_debt);
        let burn_asset;
        {
            let base_account1 = &mut ctx.accounts.base_account;
            burn_asset = base_account1 //find the asset to be burned using the token address provided in context.
            .assets
            .iter_mut()
            .find(|x| x.asset_address == *token_address)
            .unwrap();
        }
        msg!("asset to be burned inside burn {}", burn_asset.ticker[0]);
        let burned_shares = calculate_burned_shares(&burn_asset, &user_debt, &user_account.shares, &amount); //need to figure out the math logic here 
        msg!("Burned shares {}", burned_shares);
        msg!("User shares {}", user_account.shares);
        if burned_shares > user_account.shares { 
            let burned_amount = calculate_max_burned_in_token(burn_asset, &user_debt);
            burn_asset.supply -= burned_amount;
            { //because need to borrow mutably here & need to borrow immutatbly on line number 264
                let base_account = &mut ctx.accounts.base_account;
                base_account.shares -= user_account.shares;
            }
            user_account.shares = 0;
            let seeds = &[base_account.signer.as_ref(), &[base_account.nonce]];
            let signer = &[&seeds[..]];
            let cpi_ctx = CpiContext::from(&*ctx.accounts).with_signer(signer);
            token::burn(cpi_ctx, burned_amount);
            Ok(())
        } else {
            burn_asset.supply -= amount;
            user_account.shares -= burned_shares;
            { //because can't borrow mutably here and immutably on line 276
                let base_account = &mut ctx.accounts.base_account;
                base_account.shares -= burned_shares;
            }    
            let seeds = &[base_account.signer.as_ref(), &[base_account.nonce]];
            let signer = &[&seeds[..]];
            let cpi_ctx = CpiContext::from(&*ctx.accounts).with_signer(signer);
            token::burn(cpi_ctx, amount);
            Ok(())
        }
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub base_account:Account<'info, BaseAccount>,
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub collateral_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
    #[account(signer)]
    owner: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct StartStuffOff<'info> {
    #[account(init, payer=user, space=9000)]
    pub base_account: Account<'info, BaseAccount>, //Empty struct container that holds other custom accounts or structs. It checks ownership on deserialization.
    #[account(mut)]
    pub user:Signer<'info>, //Declares and enforces the constraint that the user sign the transaction- However, anchor doesn't fetch the data on that account.
    pub system_program: Program<'info, System>, //Program account container. It checks ownership on deserialization.
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
}

#[derive(Accounts)]
pub struct CreateUserAccount<'info> {
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Mint<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>, //Predefined struct with different fields. Used when there's nothing to deserialize like a SOL wallet or a signer.
    #[account(mut)]
    pub to: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    pub clock: Sysvar<'info, Clock>,
    #[account(signer)]
    owner: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct AddAsset<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    pub asset_address: AccountInfo<'info>,
    pub feed_address: AccountInfo<'info>,
    // pub prod_address: AccountInfo<'info>,
    // pub price_address: AccountInfo<'info>,
    #[account(signer)]
    pub admin: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    pub collateral_account: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct BurnToken<'info> {
    #[account(mut)]
    pub base_account: Account<'info, BaseAccount>,
    pub authority: AccountInfo<'info>,
    #[account(mut)]
    pub mint: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    #[account(mut)]
    pub user_token_account: AccountInfo<'info>,
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    pub clock: Sysvar<'info, Clock>,
    #[account(signer)]
    owner: AccountInfo<'info>,
}


impl<'a, 'b, 'c, 'info> From<&Mint<'info>> for CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
    fn from(accounts: &Mint<'info>) -> CpiContext<'a, 'b, 'c, 'info, MintTo<'info>> {
        let cpi_accounts = MintTo {
            mint: accounts.mint.to_account_info(),
            to: accounts.to.to_account_info(),
            authority: accounts.authority.to_account_info(),
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'a, 'b, 'c, 'info> From<&Withdraw<'info>> for CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
    fn from(accounts: &Withdraw<'info>) -> CpiContext<'a, 'b, 'c, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: accounts.collateral_account.to_account_info(),
            to: accounts.to.to_account_info(),
            authority: accounts.authority.to_account_info(),
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

impl<'a, 'b, 'c, 'info> From<&BurnToken<'info>> for CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
    fn from(accounts: &BurnToken<'info>) -> CpiContext<'a, 'b, 'c, 'info, Burn<'info>> {
        let cpi_accounts = Burn {
            mint: accounts.mint.to_account_info(),
            to: accounts.user_token_account.to_account_info(),
            authority: accounts.authority.to_account_info(),
        };
        let cpi_program = accounts.token_program.to_account_info();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

#[error]
pub enum ErrorCode {
    #[msg("Your error message")]
    ErrorType,
    #[msg("Assets is full")]
    AssetsFull,
    #[msg("Asset already exists")]
    AssetAlreadyExists,
    #[msg("Deposited zero")]
    ZeroDeposit,
    #[msg("Outdated oracle")]
    OutdatedOracle,
    #[msg("Missing Collateral token")]
    MissingCollateralToken,
    #[msg("Mint limit crossed")]
    MintLimit,
    #[msg("Wrong token not sythetic usd")]
    NotSyntheticUsd,
    #[msg("Not enough collateral")]
    WithdrawError,
    #[msg("Synthetic collateral is not supported")]
    SyntheticCollateral,
    #[msg("You are not admin of system")]
    Unauthorized,
}

#[derive(AnchorSerialize, AnchorDeserialize, PartialEq, Default, Clone)]
pub struct Asset { //Synthetify uses Pyth oracles to get accurate prices of all assets and stores them in one place for ease of use. Data related to the prices is kept inside Asset structure:
    // pub prod_address: Pubkey,
    // pub price_address: Pubkey,
    pub feed_address: Pubkey, //feed_address - address of Pyth oracle account
    pub asset_address: Pubkey,
    pub price: u64, //price - price multiplied by 10 to the power of PRICE OFFSET equaling 8
    pub last_update: u64, //the slot of the last price update
    pub supply: u64,
    pub decimals: u8,
    pub ticker: Vec<u8>,
}

#[account] 
pub struct BaseAccount {
    pub nonce: u8,
    pub signer: Pubkey,
    pub admin: Pubkey,
    pub mint_authority: Pubkey,
    pub initialized: bool,
    pub debt: u64,
    pub shares: u64,
    pub collateral_balance: u64,
    pub collateral_token: Pubkey,
    pub collateral_account: Pubkey,
    pub collateralization_level: u32,
    pub max_delay: u32,
    pub fee: u8, // should be in range 0-99
    pub assets: Vec<Asset>,
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub shares: u64,
    pub collateral: u64,
}