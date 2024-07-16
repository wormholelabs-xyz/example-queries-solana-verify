use crate::{
    error::ExampleQueriesSolanaVerifyError,
    state::{GuardianSignatures, WormholeGuardianSet},
};
use anchor_lang::{
    prelude::*,
    solana_program::{
        self, keccak, program_memory::sol_memcpy, secp256k1_recover::secp256k1_recover,
    },
};
use wormhole_query_sdk::{
    structs::{ChainSpecificQuery, ChainSpecificResponse, QueryResponse},
    MESSAGE_PREFIX, QUERY_MESSAGE_LEN,
};
use wormhole_raw_vaas::{utils::quorum, GuardianSetSig};
use wormhole_solana_consts::CORE_BRIDGE_PROGRAM_ID;

#[derive(Accounts)]
#[instruction(_bytes: Vec<u8>, guardian_set_index: u32)]
pub struct VerifyQuery<'info> {
    /// Guardian set used for signature verification (whose index should agree with the signature
    /// set account's guardian set index).
    #[account(
        seeds = [
            WormholeGuardianSet::SEED_PREFIX,
            guardian_set_index.to_be_bytes().as_ref()
            ],
            bump,
            seeds::program = CORE_BRIDGE_PROGRAM_ID
        )]
    guardian_set: Account<'info, WormholeGuardianSet>,

    /// Stores unverified guardian signatures as they are too large to fit in the instruction data.
    #[account(mut, has_one = refund_recipient, close = refund_recipient)]
    guardian_signatures: Account<'info, GuardianSignatures>,

    /// CHECK: This account is the refund recipient for the above signature_set
    #[account(address = guardian_signatures.refund_recipient)]
    refund_recipient: AccountInfo<'info>,
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

        // Compute the message hash.
        let message_hash = [
            MESSAGE_PREFIX,
            &solana_program::keccak::hashv(&[&bytes]).to_bytes(),
        ]
        .concat();

        // SECURITY: defense-in-depth, check again that these are the expected length
        require_eq!(
            message_hash.len(),
            QUERY_MESSAGE_LEN,
            ExampleQueriesSolanaVerifyError::InvalidMessageHash
        );

        let guardian_signatures = &ctx.accounts.guardian_signatures.guardian_signatures;

        // This section is borrowed from https://github.com/wormhole-foundation/wormhole/blob/wen/solana-rewrite/solana/programs/core-bridge/src/processor/parse_and_verify_vaa/verify_encoded_vaa_v1.rs#L72-L103
        // Also similarly used here https://github.com/pyth-network/pyth-crosschain/blob/6771c2c6998f53effee9247347cb0ac71612b3dc/target_chains/solana/programs/pyth-solana-receiver/src/lib.rs#L121-L159
        // Do we have enough signatures for quorum?
        let guardian_keys = &guardian_set.keys;
        let quorum = quorum(guardian_keys.len());
        require!(
            guardian_signatures.len() >= quorum,
            ExampleQueriesSolanaVerifyError::NoQuorum
        );

        let digest = keccak::hash(message_hash.as_slice());

        // Verify signatures
        let mut last_guardian_index = None;
        for sig_bytes in guardian_signatures {
            let sig = GuardianSetSig::try_from(sig_bytes.as_slice())
                .map_err(|_| ExampleQueriesSolanaVerifyError::InvalidSignature)?;
            // We do not allow for non-increasing guardian signature indices.
            let index = usize::from(sig.guardian_index());
            if let Some(last_index) = last_guardian_index {
                require!(
                    index > last_index,
                    ExampleQueriesSolanaVerifyError::InvalidGuardianIndex
                );
            }

            // Does this guardian index exist in this guardian set?
            let guardian_pubkey = guardian_keys
                .get(index)
                .ok_or_else(|| error!(ExampleQueriesSolanaVerifyError::InvalidGuardianIndex))?;

            // Now verify that the signature agrees with the expected Guardian's pubkey.
            verify_guardian_signature(&sig, guardian_pubkey, digest.as_ref())?;

            last_guardian_index = Some(index);
        }
        // End borrowed section

        // Done.
        Ok(())
    }
}

#[access_control(VerifyQuery::constraints(&ctx, &bytes))]
pub fn verify_query(
    ctx: Context<VerifyQuery>,
    bytes: Vec<u8>,
    _guardian_set_index: u32,
) -> Result<()> {
    let response = QueryResponse::deserialize(&bytes)
        .map_err(|_| ExampleQueriesSolanaVerifyError::FailedToParseResponse)?;
    msg!(
        "response: version: {}, req_chain: {}, req_id: {:?}, req_version: {}, req_nonce: {}, reqs_len: {}, resp_len: {}",
        response.version,
        response.request_chain_id,
        response.request_id,
        response.request.version,
        response.request.nonce,
        response.request.requests.len(),
        response.responses.len()
    );
    for idx in 0..response.request.requests.len() {
        let request = &response.request.requests[idx];
        match &request.query {
            ChainSpecificQuery::EthCallQueryRequest(q) => {
                msg!(
                    "EthCallQueryRequest: {}, {}, {}",
                    request.chain_id,
                    q.block_tag,
                    q.call_data.len()
                );
                for call_idx in 0..q.call_data.len() {
                    let call = &q.call_data[call_idx];
                    msg!("call: {:?}, {:?}", call.to, call.data)
                }
            }
            ChainSpecificQuery::EthCallByTimestampQueryRequest(_) => {
                msg!("EthCallByTimestampQueryRequest")
            }
            ChainSpecificQuery::EthCallWithFinalityQueryRequest(_) => {
                msg!("EthCallWithFinalityQueryRequest")
            }
            ChainSpecificQuery::SolanaAccountQueryRequest(_) => {
                msg!("SolanaAccountQueryRequest")
            }
        }
    }
    for idx in 0..response.responses.len() {
        let response = &response.responses[idx];
        match &response.response {
            ChainSpecificResponse::EthCallQueryResponse(eth_response) => {
                msg!(
                    "EthCallQueryResponse: {}, {}, {:?}. {}, {}",
                    response.chain_id,
                    eth_response.block_number,
                    eth_response.block_hash,
                    eth_response.block_time,
                    eth_response.results.len()
                );
                for result_idx in 0..eth_response.results.len() {
                    let result = &eth_response.results[result_idx];
                    msg!("result: {:?}", result)
                }
            }
            ChainSpecificResponse::EthCallByTimestampQueryResponse(_) => {
                msg!("EthCallByTimestampQueryResponse")
            }
            ChainSpecificResponse::EthCallWithFinalityQueryResponse(_) => {
                msg!("EthCallWithFinalityQueryResponse")
            }
            ChainSpecificResponse::SolanaAccountQueryResponse(_) => {
                msg!("SolanaAccountQueryResponse")
            }
        }
    }

    // Done.
    Ok(())
}

/**
 * Borrowed from https://github.com/wormhole-foundation/wormhole/blob/wen/solana-rewrite/solana/programs/core-bridge/src/processor/parse_and_verify_vaa/verify_encoded_vaa_v1.rs#L121
 * Also used here https://github.com/pyth-network/pyth-crosschain/blob/6771c2c6998f53effee9247347cb0ac71612b3dc/target_chains/solana/programs/pyth-solana-receiver/src/lib.rs#L432
 */
fn verify_guardian_signature(
    sig: &GuardianSetSig,
    guardian_pubkey: &[u8; 20],
    digest: &[u8],
) -> Result<()> {
    // Recover using `solana_program::secp256k1_recover`. Public key recovery costs 25k compute
    // units. And hashing this public key to recover the Ethereum public key costs about 13k.
    let recovered = {
        // Recover EC public key (64 bytes).
        let pubkey = secp256k1_recover(digest, sig.recovery_id(), &sig.rs())
            .map_err(|_| ExampleQueriesSolanaVerifyError::InvalidSignature)?;

        // The Ethereum public key is the last 20 bytes of keccak hashed public key above.
        let hashed = keccak::hash(&pubkey.to_bytes());

        let mut eth_pubkey = [0; 20];
        sol_memcpy(&mut eth_pubkey, &hashed.0[12..], 20);

        eth_pubkey
    };

    // The recovered public key should agree with the Guardian's public key at this index.
    require!(
        recovered == *guardian_pubkey,
        ExampleQueriesSolanaVerifyError::InvalidGuardianKeyRecovery
    );

    // Done.
    Ok(())
}
