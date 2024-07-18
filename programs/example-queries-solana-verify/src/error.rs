//! Errors that may arise when interacting with the Example Queries Solana Verify Program.

use anchor_lang::prelude::error_code;

/// * \>= 0x100  -- Query Verification.
///
/// NOTE: All of these error codes when triggered are offset by `ERROR_CODE_OFFSET` (6000). So for
/// example, `InvalidMessageHash` will return as 6256.
#[error_code]
pub enum ExampleQueriesSolanaVerifyError {
    #[msg("WriteAuthorityMismatch")]
    WriteAuthorityMismatch = 0x100,

    #[msg("GuardianSetExpired")]
    GuardianSetExpired = 0x101,

    #[msg("InvalidMessageHash")]
    InvalidMessageHash = 0x102,

    #[msg("NoQuorum")]
    NoQuorum = 0x103,

    #[msg("InvalidGuardianIndexNonIncreasing")]
    InvalidGuardianIndexNonIncreasing = 0x104,

    #[msg("InvalidGuardianIndexOutOfRange")]
    InvalidGuardianIndexOutOfRange = 0x105,

    #[msg("InvalidSignature")]
    InvalidSignature = 0x106,

    #[msg("InvalidGuardianKeyRecovery")]
    InvalidGuardianKeyRecovery = 0x107,

    #[msg("FailedToParseResponse")]
    FailedToParseResponse = 0x110,
}
