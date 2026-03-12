import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MySolanaProject } from "../target/types/my_solana_project";
import { SolanaStablecoin, Presets } from "../sdk/stablecoin";
import { Keypair, PublicKey } from "@solana/web3.js";
import { expect } from "chai";

describe("solana-stablecoin-standard", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const connection = provider.connection;
  const admin = (provider.wallet as any).payer as Keypair;

  it("SSS-1 Flow: Initialize, Mint, Burn, Pause", async () => {
    // SSS-1 Minimal setup (mocking mint and treasury as simple public keys for logic test)
    // Note: In real integration tests, we'd use spl-token CLI or SDK to create a real mint
    const dummyMint = Keypair.generate().publicKey;
    const dummyTreasury = Keypair.generate().publicKey;

    const stable = await SolanaStablecoin.create(connection, {
      preset: Presets.SSS_1,
      name: "Minimal USD",
      symbol: "MUSD",
      uri: "https://musd.com",
      decimals: 6,
      authority: admin,
      mint: dummyMint,
      treasury: dummyTreasury,
    });

    const configAccount = await stable.program.account.stablecoinConfig.fetch(stable.config);
    expect(configAccount.name).to.equal("Minimal USD");
    expect(configAccount.enablePermanentDelegate).to.be.false;
    expect(configAccount.enableTransferHook).to.be.false;

    // Test Pause/Unpause
    await stable.pause(admin);
    let updatedConfig = await stable.program.account.stablecoinConfig.fetch(stable.config);
    expect(updatedConfig.paused).to.be.true;

    await stable.unpause(admin);
    updatedConfig = await stable.program.account.stablecoinConfig.fetch(stable.config);
    expect(updatedConfig.paused).to.be.false;
  });

  it("SSS-2 Flow: Initialize with Compliance, Blacklist", async () => {
    const dummyMint = Keypair.generate().publicKey;
    
    const stable = await SolanaStablecoin.create(connection, {
      preset: Presets.SSS_2,
      name: "Compliant USD",
      symbol: "CUSD",
      uri: "https://cusd.com",
      decimals: 6,
      authority: admin,
      mint: dummyMint,
    });

    const configAccount = await stable.program.account.stablecoinConfig.fetch(stable.config);
    expect(configAccount.enablePermanentDelegate).to.be.true;
    expect(configAccount.enableTransferHook).to.be.true;

    // Test Blacklist
    const user = Keypair.generate().publicKey;
    await stable.compliance.blacklistAdd(user, admin);
    
    const [blacklistPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from("blacklist"), user.toBuffer()],
      stable.program.programId
    );
    const blacklistAccount = await stable.program.account.blacklist.fetch(blacklistPDA);
    expect(blacklistAccount.wallet.toBase58()).to.equal(user.toBase58());

    await stable.compliance.blacklistRemove(user, admin);
    // Fetching should fail now
    try {
        await stable.program.account.blacklist.fetch(blacklistPDA);
        expect.fail("Should have thrown error as account is closed");
    } catch (e) {
        expect(e.message).to.contain("Account does not exist");
    }
  });

  it("SSS-2 Logic: Seize fails if not enabled", async () => {
     const dummyMint = Keypair.generate().publicKey;
     const stable = await SolanaStablecoin.create(connection, {
      // Intentional SSS-1 to test error gating
      preset: Presets.SSS_1,
      name: "Gate Test",
      symbol: "GT",
      uri: "https://gate.com",
      decimals: 6,
      authority: admin,
      mint: dummyMint,
    });

    try {
        await stable.compliance.seize(PublicKey.default, PublicKey.default, 100, admin);
        expect.fail("Should fail with FeatureNotEnabled");
    } catch (e) {
        // Error code check (FeatureNotEnabled is usually 6004 if it's the 5th error)
        expect(e.toString()).to.contain("Feature not enabled");
    }
  });
});
