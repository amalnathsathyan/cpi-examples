use crate::dlmm;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DlmmAddLiquidityOneSide<'info> {
    #[account(mut)]
    /// CHECK: The user's position account
    pub position: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The pool account. Must match the lb_pair stored inside position,
    /// bin_array_bitmap_extension, bin_array_lower, and bin_array_upper.
    pub lb_pair: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: Bin array bitmap extension account of the pool. Only required
    /// when the active bin falls outside the main bitmap range (|bin_id| > 512).
    /// Pass None if not needed.
    pub bin_array_bitmap_extension: Option<UncheckedAccount<'info>>,

    #[account(mut)]
    /// CHECK: User token account for the token being deposited (either token X or Y).
    /// Tokens are transferred FROM this account into the pool reserve.
    pub user_token: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The pool's reserve vault for the token being deposited.
    /// Use lb_pair.reserve_x for token X deposits, lb_pair.reserve_y for token Y.
    pub reserve: UncheckedAccount<'info>,

    /// CHECK: Mint of the token being deposited.
    /// Must match lb_pair.token_x_mint or lb_pair.token_y_mint.
    pub token_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The lower bin array account covering the position's bin range.
    /// PDA: ["bin_array", lb_pair, floor(lower_bin_id / 70)]
    pub bin_array_lower: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The upper bin array account covering the position's bin range.
    /// PDA: ["bin_array", lb_pair, floor(upper_bin_id / 70)]
    /// May be the same account as bin_array_lower if the position fits in one array.
    pub bin_array_upper: UncheckedAccount<'info>,

    /// CHECK: The authority that owns user_token. Must sign the transaction.
    pub sender: Signer<'info>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI.
    /// PDA derived as: find_program_address(&[b"__event_authority"], &dlmm::ID)
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Token program of the mint being deposited.
    /// Use Token (spl-token) or Token-2022 depending on the pool's token program.
    pub token_program: UncheckedAccount<'info>,
}

/// Adds single-sided liquidity to a Meteora DLMM position.
///
/// Single-sided means only one token (X or Y) is deposited, distributing
/// liquidity exclusively to bins above the active price (for token X) or
/// below (for token Y).
///
/// # Arguments
///
/// * `ctx` - The context containing all required accounts.
/// * `amount` - Total amount of the single token to deposit, in base units.
/// * `active_id` - The active bin ID observed off-chain prior to building
///   the transaction. Used to validate slippage on-chain.
/// * `max_active_bin_slippage` - Maximum allowed bin ID deviation from
///   `active_id` at execution time. Protects against price movement between
///   observation and execution. Recommended: 3–10.
/// * `bin_liquidity_dist` - Per-bin weight distribution. Each entry specifies
///   a bin_id and a relative weight (u16). The program normalises these
///   weights internally so only the ratios matter.
///
///   Rules for bin_id selection:
///   - Token X deposits: all bin_ids must be strictly > active_id
///   - Token Y deposits: all bin_ids must be <= active_id
///   - All bin_ids must fall within [position.lower_bin_id, position.upper_bin_id]
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_dlmm_add_liquidity_one_side(
    ctx: Context<DlmmAddLiquidityOneSide>,
    amount: u64,
    active_id: i32,
    max_active_bin_slippage: i32,
    bin_liquidity_dist: Vec<dlmm::types::BinLiquidityDistributionByWeight>,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::AddLiquidityOneSide {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        user_token: ctx.accounts.user_token.to_account_info(),
        reserve: ctx.accounts.reserve.to_account_info(),
        token_mint: ctx.accounts.token_mint.to_account_info(),
        bin_array_lower: ctx.accounts.bin_array_lower.to_account_info(),
        bin_array_upper: ctx.accounts.bin_array_upper.to_account_info(),
        sender: ctx.accounts.sender.to_account_info(),
        token_program: ctx.accounts.token_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let liquidity_parameter = dlmm::types::LiquidityOneSideParameter {
        amount,
        active_id,
        max_active_bin_slippage,
        bin_liquidity_dist,
    };

    let cpi_context =
        CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts);

    dlmm::cpi::add_liquidity_one_side(cpi_context, liquidity_parameter)
}