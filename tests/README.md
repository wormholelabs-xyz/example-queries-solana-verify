# TypeScript Anchor Tests

The goal of these tests is to provide positive and negative cases for account annotations and custom errors in the Example Queries Solana Verify program.

- [x] [post_signatures](/programs/example-queries-solana-verify/src/instructions/post_signatures.rs)
  - [x] Successfully posts signatures
  - [x] Successfully appends signatures
  - [x] Rejects append by non-initial payer
- [x] [verify_query](/programs/example-queries-solana-verify/src/instructions/verify_query.rs)
  - [x] Successfully verifies mainnet queries
  - [x] Successfully verifies mock queries
  - [x] Successfully closed the signature set
  - [x] Rejects guardian set account not owned by the core bridge
  - [x] Rejects guardian set account mismatch
  - [x] Rejects refund recipient account mismatch
  - [x] Rejects expired guardian set
  - [x] Rejects no quorum
  - [x] Rejects out of order guardian signatures
  - [x] Rejects duplicate guardian signatures
  - [x] Rejects guardian index out of bounds
  - [x] Rejects invalid signature
  - [x] Rejects invalid message hash
  - [x] Rejects un-parse-able response
- [x] [close_signatures](/programs/example-queries-solana-verify/src/instructions/close_signatures.rs)
  - [x] Successfully closes signature accounts
  - [x] Rejects refund recipient account mismatch
  - [x] Rejects without refund recipient as signer
