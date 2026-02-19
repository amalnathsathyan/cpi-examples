use crate::dlmm;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DlmmRemoveAllLiquidity<'info> {
    #[account(mut)]
    /// CHECK: The user's position account. After this instruction, the position
    /// will have zero liquidity across all bins but the account remains open.
    /// Call close_position subsequently to reclaim rent.
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
    /// CHECK: User token account to receive all withdrawn token X.
    pub user_token_x: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: User token account to receive all withdrawn token Y.
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

/// Removes 100% of liquidity from all bins in a Meteora DLMM position in one shot.
///
/// This is the recommended way to fully exit a position before closing it.
/// After this call, all tokens X and Y are returned to the user's token accounts
/// and every bin in the position has zero liquidity.
///
/// NOTE: This does NOT close the position account or claim accumulated fees.
/// The full exit sequence is:
///   1. remove_all_liquidity  — drain all bins, return tokens
///   2. claim_fee             — claim any accumulated swap fees
///   3. close_position        — close the position account, reclaim rent SOL
///
/// # Arguments
///
/// * `ctx` - The context containing all required accounts.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_dlmm_remove_all_liquidity(ctx: Context<DlmmRemoveAllLiquidity>) -> Result<()> {
    let accounts = dlmm::cpi::accounts::RemoveAllLiquidity {
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

    dlmm::cpi::remove_all_liquidity(cpi_context)
}