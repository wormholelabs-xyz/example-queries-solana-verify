// modified from https://github.com/wormhole-foundation/wormhole/blob/main/sdk/js/src/solana/wormhole/instructions/verifySignature.ts
import * as anchor from "@coral-xyz/anchor";
import { getWormholeBridgeData } from "./config";
import { keccak256 } from "@ethersproject/keccak256";
import { QUERY_RESPONSE_PREFIX } from "@wormhole-foundation/wormhole-query-sdk";
import { deriveGuardianSetKey, getGuardianSet } from "./guardianSet";
import { responseSignaturesToGuardianSignature } from "./GuardianSignature";
import { createSecp256k1Instruction } from "./secp256k1";
import { ExampleQueriesSolanaVerify } from "../../target/types/example_queries_solana_verify";

export async function createVerifySignaturesInstructions(
  connection: anchor.web3.Connection,
  program: anchor.Program<ExampleQueriesSolanaVerify>,
  wormholeProgramId: anchor.web3.PublicKey,
  payer: anchor.web3.PublicKey,
  bytes: string,
  signatures: string[],
  signatureSet: anchor.web3.PublicKey,
  commitment?: anchor.web3.Commitment
): Promise<anchor.web3.TransactionInstruction[]> {
  const MAX_LEN_GUARDIAN_KEYS = 19;

  const hash = Buffer.concat([
    Buffer.from(QUERY_RESPONSE_PREFIX),
    Buffer.from(keccak256(Buffer.from(bytes, "hex")).slice(2), "hex"),
  ]);

  const info = await getWormholeBridgeData(
    connection,
    wormholeProgramId,
    commitment
  );
  const guardianSetIndex = info.guardianSetIndex;

  const guardianSetData = await getGuardianSet(
    connection,
    wormholeProgramId,
    guardianSetIndex,
    commitment
  );

  const guardianSignatures = responseSignaturesToGuardianSignature(signatures);
  const guardianKeys = guardianSetData.keys;

  const batchSize = 7;
  const instructions: anchor.web3.TransactionInstruction[] = [];
  for (let i = 0; i < Math.ceil(guardianSignatures.length / batchSize); ++i) {
    const start = i * batchSize;
    const end = Math.min(guardianSignatures.length, (i + 1) * batchSize);

    const signatureStatus = new Array(MAX_LEN_GUARDIAN_KEYS).fill(-1);
    const signatures: Buffer[] = [];
    const keys: Buffer[] = [];
    for (let j = 0; j < end - start; ++j) {
      const item = guardianSignatures.at(j + start)!;
      signatures.push(item.signature);

      const key = guardianKeys.at(item.index)!;
      keys.push(key);

      signatureStatus[item.index] = j;
    }

    instructions.push(createSecp256k1Instruction(signatures, keys, hash));

    const ix = await program.methods
      .verifySignatures(signatureStatus)
      .accounts({
        payer,
        guardianSet: deriveGuardianSetKey(wormholeProgramId, guardianSetIndex),
        instructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
        signatureSet,
      })
      .instruction();

    instructions.push(ix);
  }
  return instructions;
}
