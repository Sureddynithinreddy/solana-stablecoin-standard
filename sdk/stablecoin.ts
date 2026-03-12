import * as anchor from "@coral-xyz/anchor";
import { AnchorProvider, Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import idl from "./my_solana_project.json";
import { STABLECOIN_PROGRAM_ID } from "./constants";

export class StablecoinSDK {

  program: Program;

  constructor(provider: AnchorProvider) {
    this.program = new Program(idl as any, STABLECOIN_PROGRAM_ID, provider);
  }

  async mintTokens(
    admin: PublicKey,
    mint: PublicKey,
    userTokenAccount: PublicKey,
    amount: number
  ) {

    return await this.program.methods
      .mintTokens(new anchor.BN(amount))
      .accounts({
        admin,
        mint,
        userTokenAccount,
      })
      .rpc();
  }

}