use anchor_lang::prelude::*;

declare_id!("HkDXBFRS9Tv9295d9wEVRL61c1pUXj3WZHiaTNZ9Q7TQ");

pub mod error;

mod processor;
pub(crate) use processor::*;

pub mod state;

#[program]
pub mod example_queries_solana_verify {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn verify_signatures(
        ctx: Context<VerifySignatures>,
        signer_indices: [i8; 19],
    ) -> Result<()> {
        processor::verify_signatures(ctx, signer_indices)
    }

    pub fn verify_query(ctx: Context<VerifyQuery>, bytes: Vec<u8>) -> Result<()> {
        processor::verify_query(ctx, bytes)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
