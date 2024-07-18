use anchor_lang::prelude::*;

declare_id!("HkDXBFRS9Tv9295d9wEVRL61c1pUXj3WZHiaTNZ9Q7TQ");

pub mod error;

mod instructions;
pub(crate) use instructions::*;

pub mod state;

#[program]
pub mod example_queries_solana_verify {
    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn close_signatures(ctx: Context<CloseSignatures>) -> Result<()> {
        instructions::close_signatures(ctx)
    }

    pub fn post_signatures(
        ctx: Context<PostSignatures>,
        guardian_signatures: Vec<[u8; 66]>,
        total_signatures: u8,
    ) -> Result<()> {
        instructions::post_signatures(ctx, guardian_signatures, total_signatures)
    }

    pub fn verify_query(
        ctx: Context<VerifyQuery>,
        bytes: Vec<u8>,
        guardian_set_index: u32,
    ) -> Result<()> {
        instructions::verify_query(ctx, bytes, guardian_set_index)
    }
}

#[derive(Accounts)]
pub struct Initialize {}
