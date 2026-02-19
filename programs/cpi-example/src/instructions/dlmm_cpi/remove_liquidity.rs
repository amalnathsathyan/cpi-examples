use crate::dlmm;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DlmmRemoveLiquidity<'info> {
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
    /// CHECK: User token account to receive withdrawn token X.
    pub user_token_x: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: User token account to receive withdrawn token Y.
    pub user_token_y: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The pool's reserve vault for token X. Derived from lb_pair.reserve_x.
    pub reserve_x: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The pool's reserve vault for token Y. Derived from lb_pair.reserve_y.
    pub reserve_y: UncheckedAccount<'info>,

    /// CHECK: Mint of token X. Must match lb_pair.token_x_mint.
    pub token_x_mint: UncheckedAccount<'info>,

    /// CHECK: Mint of token Y. Must match lb_pair.token_y_mint.
    pub token_y_mint: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The lower bin array account covering the position's bin range.
    /// PDA: ["bin_array", lb_pair, floor(lower_bin_id / 70)]
    pub bin_array_lower: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The upper bin array account covering the position's bin range.
    /// PDA: ["bin_array", lb_pair, floor(upper_bin_id / 70)]
    /// May be the same account as bin_array_lower if the position fits in one array.
    pub bin_array_upper: UncheckedAccount<'info>,

    /// CHECK: The authority that owns the position. Must sign the transaction.
    pub sender: Signer<'info>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI.
    /// PDA derived as: find_program_address(&[b"__event_authority"], &dlmm::ID)
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Token program of token X mint.
    /// Use Token (spl-token) or Token-2022 depending on the pool's token program.
    pub token_x_program: UncheckedAccount<'info>,

    /// CHECK: Token program of token Y mint.
    /// Use Token (spl-token) or Token-2022 depending on the pool's token program.
    pub token_y_program: UncheckedAccount<'info>,
}

/// Removes liquidity from specific bins in a Meteora DLMM position.
///
/// Unlike `remove_all_liquidity`, this instruction gives surgical control —
/// you specify exactly which bins to withdraw from and by what percentage.
/// The position account remains open after this call; use `close_position`
/// only after all bins have been fully drained.
///
/// # Arguments
///
/// * `ctx` - The context containing all required accounts.
/// * `bin_liquidity_removal` - A list of per-bin removal instructions. Each entry
///   specifies a `bin_id` and `bps_to_remove` (basis points out of 10000):
///   - 10000 bps = 100% (full removal from that bin)
///   -  5000 bps =  50% (partial removal from that bin)
///   Only bins listed here are affected; unlisted bins are untouched.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_dlmm_remove_liquidity(
    ctx: Context<DlmmRemoveLiquidity>,
    bin_liquidity_removal: Vec<dlmm::types::BinLiquidityReduction>,
) -> Result<()> {
    let accounts = dlmm::cpi::accounts::RemoveLiquidity {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        bin_array_bitmap_extension: ctx
            .accounts
            .bin_array_bitmap_extension
            .as_ref()
            .map(|account| account.to_account_info()),
        user_token_x: ctx.accounts.user_token_x.to_account_info(),
        user_token_y: ctx.accounts.user_token_y.to_account_info(),
        reserve_x: ctx.accounts.reserve_x.to_account_info(),
        reserve_y: ctx.accounts.reserve_y.to_account_info(),
        token_x_mint: ctx.accounts.token_x_mint.to_account_info(),
        token_y_mint: ctx.accounts.token_y_mint.to_account_info(),
        bin_array_lower: ctx.accounts.bin_array_lower.to_account_info(),
        bin_array_upper: ctx.accounts.bin_array_upper.to_account_info(),
        sender: ctx.accounts.sender.to_account_info(),
        token_x_program: ctx.accounts.token_x_program.to_account_info(),
        token_y_program: ctx.accounts.token_y_program.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let cpi_context =
        CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts);

    dlmm::cpi::remove_liquidity(cpi_context, bin_liquidity_removal)
}