# Queries Verification on Solana PoC

This is a demo of verifying and parsing [Wormhole Queries](https://wormhole.com/queries/) on Solana.

This project was made with [Anchor](https://www.anchor-lang.com/).

Learn more about developing with Queries in [the docs](https://docs.wormhole.com/wormhole/queries/getting-started).

## Accounts

- [GuardianSignatures](programs/solana-world-id-program/src/state/guardian_signatures.rs) stores unverified guardian signatures for subsequent verification. These are created with `post_signatures` in service of verifying a root via Queries and closed when that root is verified with `verify_query` or can be explicitly closed with `close_signatures` by the initial payer.

## Instructions

- [post_signatures](programs/example-queries-solana-verify/src/instructions/post_signatures.rs) posts unverified guardian signatures for verification during `update_root_with_query`.
- [verify_query](programs/example-queries-solana-verify/src/instructions/verify_query.rs) with a Query response and `GuardianSignatures` account, verifies the signatures against an active guardian set and logs the Query response. This is where you would add additional verification relevant to your use case and process the result.
- [close_signatures](programs/example-queries-solana-verify/src/instructions/close_signatures.rs) allows the initial payer to close a `GuardianSignatures` account in case the query was invalid.

## Testing

```bash
anchor test
```

## Building

### Wormhole Testnet / Solana Devnet

```bash
anchor build -- --no-default-features --features testnet
```

### Mainnet

```bash
anchor build
```

---

⚠ **This software is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
implied. See the License for the specific language governing permissions and limitations under the License.**
