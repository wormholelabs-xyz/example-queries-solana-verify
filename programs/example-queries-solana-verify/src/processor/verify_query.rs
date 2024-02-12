use crate::{
    error::ExampleQueriesSolanaVerifyError,
    state::{QuerySignatureSet, WormholeGuardianSet},
    MESSAGE_PREFIX, QUERY_MESSAGE_LEN,
};
use anchor_lang::{
    prelude::*,
    solana_program::{self},
};

/// Compute quorum based on the number of guardians in a guardian set.
#[inline]
pub fn quorum(num_guardians: usize) -> usize {
    (2 * num_guardians) / 3 + 1
}

#[derive(Accounts)]
pub struct VerifyQuery<'info> {
    /// Guardian set used for signature verification (whose index should agree with the signature
    /// set account's guardian set index).
    #[account(
        seeds = [
            WormholeGuardianSet::SEED_PREFIX,
            signature_set.guardian_set_index.to_be_bytes().as_ref()
        ],
        bump,
        seeds::program = wormhole_anchor_sdk::wormhole::program::id()
    )]
    guardian_set: Account<'info, WormholeGuardianSet>,

    /// Stores signature validation from Sig Verify native program.
    signature_set: Account<'info, QuerySignatureSet>,
}

impl<'info> VerifyQuery<'info> {
    pub fn constraints(ctx: &Context<Self>, bytes: &Vec<u8>) -> Result<()> {
        let guardian_set = ctx.accounts.guardian_set.clone().into_inner();

        // Check that the guardian set is still active.
        let timestamp = Clock::get()?
            .unix_timestamp
            .try_into()
            .expect("timestamp overflow");
        require!(
            guardian_set.is_active(&timestamp),
            ExampleQueriesSolanaVerifyError::GuardianSetExpired
        );

        let signature_set = &ctx.accounts.signature_set;

        // Number of verified signatures in the signature set account must be at least quorum with
        // the guardian set.
        require!(
            signature_set.num_verified() >= quorum(guardian_set.keys.len()),
            ExampleQueriesSolanaVerifyError::NoQuorum
        );

        // Recompute the message hash and compare it to the one in the signature set account.
        let recomputed = [
            MESSAGE_PREFIX,
            &solana_program::keccak::hashv(&[&bytes]).to_bytes(),
        ]
        .concat();

        // And verify that the message hash is the same as the one already encoded in the signature
        // set.
        require!(
            recomputed == signature_set.message,
            ExampleQueriesSolanaVerifyError::InvalidMessageHash
        );

        // SECURITY: defense-in-depth, check again that these are the expected length
        require_eq!(
            recomputed.len(),
            QUERY_MESSAGE_LEN,
            ExampleQueriesSolanaVerifyError::InvalidSigVerifyInstruction
        );

        // Done.
        Ok(())
    }
}

#[access_control(VerifyQuery::constraints(&ctx, &bytes))]
pub fn verify_query(ctx: Context<VerifyQuery>, bytes: Vec<u8>) -> Result<()> {
    // Done.
    Ok(())
}
