use crate::dlmm;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct DlmmClosePosition<'info> {
    #[account(mut)]
    /// CHECK: The user's position account to be closed. Must have zero
    /// liquidity across all bins before this instruction can succeed.
    pub position: UncheckedAccount<'info>,

    #[account(mut)]
    /// CHECK: The pool account. Must match the lb_pair stored inside
    /// position, bin_array_lower, and bin_array_upper.
    pub lb_pair: UncheckedAccount<'info>,

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

    #[account(mut)]
    /// CHECK: The account that will receive the reclaimed rent lamports
    /// from closing the position account. Typically the user's wallet.
    pub rent_receiver: UncheckedAccount<'info>,

    #[account(address = dlmm::ID)]
    /// CHECK: DLMM program
    pub dlmm_program: UncheckedAccount<'info>,

    /// CHECK: DLMM program event authority for event CPI.
    /// PDA derived as: find_program_address(&[b"__event_authority"], &dlmm::ID)
    pub event_authority: UncheckedAccount<'info>,
}

/// Closes a Meteora DLMM position and reclaims rent.
///
/// The position must have all liquidity removed (via `remove_liquidity` or
/// `remove_all_liquidity`) and all fees claimed before this will succeed.
/// Once closed, the rent lamports are returned to `rent_receiver`.
///
/// # Arguments
///
/// * `ctx` - The context containing all required accounts.
///
/// # Returns
///
/// Returns a `Result` indicating success or failure.
pub fn handle_dlmm_close_position(ctx: Context<DlmmClosePosition>) -> Result<()> {
    let accounts = dlmm::cpi::accounts::ClosePosition {
        position: ctx.accounts.position.to_account_info(),
        lb_pair: ctx.accounts.lb_pair.to_account_info(),
        bin_array_lower: ctx.accounts.bin_array_lower.to_account_info(),
        bin_array_upper: ctx.accounts.bin_array_upper.to_account_info(),
        sender: ctx.accounts.sender.to_account_info(),
        rent_receiver: ctx.accounts.rent_receiver.to_account_info(),
        event_authority: ctx.accounts.event_authority.to_account_info(),
        program: ctx.accounts.dlmm_program.to_account_info(),
    };

    let cpi_context =
        CpiContext::new(ctx.accounts.dlmm_program.to_account_info(), accounts);

    dlmm::cpi::close_position(cpi_context)
}