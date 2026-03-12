# Operations Runbook

This document provides the complete operator runbook for managing the Solana Stablecoin Standard deployment. It covers routine operations, emergency procedures, and all compliance-gated functions.

---

## Prerequisites

### Required Tools

| Tool | Version | Purpose |
|---|---|---|
| `solana-cli` | ≥ 1.18 | Wallet management, network queries |
| `anchor-cli` | ≥ 0.29 | Program build, deploy, test |
| `node` / `npm` | ≥ 18 | SDK execution |
| `ts-node` | ≥ 10 | Run TypeScript scripts |

```bash
# Verify installations
solana --version
anchor --version
node --version
```

### Access Requirements

| Role | Requirement |
|---|---|
| **Admin** | Admin private keypair (JSON file or hardware wallet) |
| **Minter** | Minter private keypair |
| **Read-only** | Any RPC URL, no keypair required |

### Environment Setup

```bash
# Set cluster to devnet
solana config set --url devnet

# Verify admin keypair
solana-keygen pubkey ~/.config/solana/admin.json

# Fund admin wallet (devnet)
solana airdrop 2 $(solana-keygen pubkey ~/.config/solana/admin.json)
```

---

## Routine Operations

### Daily Health Check

```bash
# 1. Verify protocol status
solana program show CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh

# 2. Check recent program activity
solana transaction-history CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh --limit 20

# 3. Check token supply
spl-token supply Henxk2RfJY2UqkiihsiT3fqHPLrwHnJ68ejfVVBtYBFN
```

### Check Protocol Pause Status

```typescript
import * as anchor from "@coral-xyz/anchor";
import idl from "./sdk/my_solana_project.json";
import { STABLECOIN_PROGRAM_ID } from "./sdk/constants";

const provider = anchor.AnchorProvider.env();
const program = new anchor.Program(idl as any, STABLECOIN_PROGRAM_ID, provider);

const config = await program.account.stablecoinConfig.fetch(CONFIG_PUBKEY);
console.log("Protocol paused:", config.paused);
console.log("Admin:", config.admin.toString());
console.log("Minter:", config.minter.toString());
```

---

## Minting Operations

### Standard Mint

> **Authority required:** Minter keypair (`config.minter`)

```typescript
import * as anchor from "@coral-xyz/anchor";
import { StablecoinSDK } from "./sdk";

const provider = anchor.AnchorProvider.env();
const sdk = new StablecoinSDK(provider);

// Mint 1,000 USDS (6 decimals → 1_000_000_000 units)
const txSig = await sdk.mintTokens(
  minterKeypair.publicKey,           // minter authority
  new PublicKey("Henxk2RfJY2Uqkiihsi..."), // mint address
  userTokenAccount,                  // recipient token account
  1_000_000_000                      // amount in base units
);

console.log("Mint transaction:", txSig);
```

### Bulk Mint

```typescript
const recipients = [
  { account: user1TokenAccount, amount: 1_000_000 },
  { account: user2TokenAccount, amount: 5_000_000 },
  { account: user3TokenAccount, amount: 10_000_000 },
];

for (const r of recipients) {
  const tx = await sdk.mintTokens(minter.publicKey, mint, r.account, r.amount);
  console.log(`Minted to ${r.account.toString()} - tx: ${tx}`);
  await new Promise(res => setTimeout(res, 800)); // rate limit
}
```

### Emergency Mint Halt (No Protocol Pause)

If you need to stop minting without pausing the whole protocol, update the minter to an unusable address:

```typescript
// Set minter to system program (effectively disables minting)
await program.methods
  .updateMinter(new PublicKey("11111111111111111111111111111112"))
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();

// To re-enable, set a real minter again
await program.methods
  .updateMinter(newMinterKeypair.publicKey)
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();
```

---

## Protocol Pause / Unpause

> **Authority required:** Admin keypair  
> **Effect:** Blocks all `mint_tokens` calls while paused

### When to Pause

- Suspected security breach or exploit
- Regulatory investigation requiring halt
- Smart contract bug discovered
- Abnormal market conditions requiring manual review

### Pause Procedure

```typescript
// Step 1: Pause
await program.methods
  .pause()
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();

// Step 2: Verify
const config = await program.account.stablecoinConfig.fetch(configPubkey);
console.assert(config.paused === true, "Pause failed!");

// Step 3: Notify stakeholders
// - Email compliance@company.com
// - Update status page
// - Alert exchange partners
```

### Unpause Procedure

```typescript
// Step 1: Confirm incident resolved + obtain approval
// Step 2: Unpause
await program.methods
  .unpause()
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();

// Step 3: Verify
const config = await program.account.stablecoinConfig.fetch(configPubkey);
console.assert(config.paused === false, "Unpause failed!");
```

---

## Freeze / Thaw Account

> **Authority required:** Admin keypair  
> **Effect:** Frozen accounts cannot send or receive tokens

### Freeze Procedure

```typescript
// 1. Identify the user's token account (not their wallet)
const userMint = new PublicKey("Henxk2RfJY2UqkiihsiT3fqHPLrwHnJ68ejfVVBtYBFN");
const userWallet = new PublicKey("UserWalletAddress...");
const userTokenAccount = await getAssociatedTokenAddress(userMint, userWallet, false, TOKEN_2022_PROGRAM_ID);

// 2. Freeze
await program.methods
  .freezeAccount()
  .accounts({
    config: configPubkey,
    admin: adminKeypair.publicKey,
    mint,
    userTokenAccount,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
  })
  .signers([adminKeypair])
  .rpc();

// 3. Log action
console.log(`ACCOUNT_FROZEN: ${userWallet.toString()} at ${new Date().toISOString()}`);
```

### Thaw Procedure

```typescript
// 1. Obtain compliance / legal approval to thaw
// 2. Execute thaw
await program.methods
  .thawAccount()
  .accounts({
    config: configPubkey,
    admin: adminKeypair.publicKey,
    mint,
    userTokenAccount,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
  })
  .signers([adminKeypair])
  .rpc();

console.log(`ACCOUNT_THAWED: ${userWallet.toString()} at ${new Date().toISOString()}`);
```

---

## Blacklist Management

> **Authority required:** Admin keypair  
> **Effect:** All transfers involving a blacklisted wallet are rejected by the Transfer Hook

### Add to Blacklist

```typescript
import { PublicKey, SystemProgram } from "@solana/web3.js";

const targetWallet = new PublicKey("WalletToBlacklist...");

// Compute PDA
const [blacklistPDA] = PublicKey.findProgramAddressSync(
  [Buffer.from("blacklist"), targetWallet.toBuffer()],
  STABLECOIN_PROGRAM_ID
);

await program.methods
  .addToBlacklist(targetWallet)
  .accounts({
    config: configPubkey,
    blacklist: blacklistPDA,
    admin: adminKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([adminKeypair])
  .rpc();

console.log(`WALLET_BLACKLISTED: ${targetWallet.toString()}`);
```

### Remove from Blacklist

```typescript
await program.methods
  .removeFromBlacklist()
  .accounts({
    config: configPubkey,
    blacklist: blacklistPDA,
    admin: adminKeypair.publicKey,
  })
  .signers([adminKeypair])
  .rpc();

console.log(`WALLET_UNBLACKLISTED: ${targetWallet.toString()}`);
```

### Verify Blacklist Status

```typescript
try {
  const blacklistAccount = await program.account.blacklist.fetch(blacklistPDA);
  console.log("BLACKLISTED:", blacklistAccount.wallet.toString());
} catch {
  console.log("Not blacklisted.");
}
```

---

## Token Seizure

> **Authority required:** Admin keypair  
> **Legal requirement:** Court order, regulatory directive, or documented fraud case  
> **Effect:** Tokens transferred from user's account to treasury

```typescript
// Prerequisites:
// 1. User account must be frozen first
// 2. Have legal authorization documented

const amountToSeize = new anchor.BN(1_000_000); // 1 USDS

await program.methods
  .seize(amountToSeize)
  .accounts({
    config: configPubkey,
    admin: adminKeypair.publicKey,
    mint,
    userTokenAccount,
    treasuryTokenAccount,
    tokenProgram: TOKEN_2022_PROGRAM_ID,
  })
  .signers([adminKeypair])
  .rpc();

console.log(`TOKENS_SEIZED: ${amountToSeize.toString()} from ${userWallet.toString()}`);
// MANDATORY: Record in the compliance audit log
```

---

## Admin Operations

### Update Minter Role

```typescript
const newMinter = Keypair.generate(); // or existing keypair

await program.methods
  .updateMinter(newMinter.publicKey)
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();

console.log(`MINTER_UPDATED: new minter = ${newMinter.publicKey.toString()}`);
```

### Transfer Admin Authority (Key Rotation)

> ⚠️ This is irreversible with the old keypair. Ensure the new admin keypair is secured first.

```typescript
const newAdmin = Keypair.generate(); // or new secure keypair

// Transfer authority
await program.methods
  .transferAuthority(newAdmin.publicKey)
  .accounts({ config: configPubkey, admin: oldAdminKeypair.publicKey })
  .signers([oldAdminKeypair])
  .rpc();

// Verify
const config = await program.account.stablecoinConfig.fetch(configPubkey);
console.assert(config.admin.equals(newAdmin.publicKey), "Authority transfer failed!");
console.log(`ADMIN_TRANSFERRED: new admin = ${newAdmin.publicKey.toString()}`);
```

---

## Deployment Operations

### Build & Deploy

```bash
# Build programs
anchor build

# Run tests (devnet)
anchor test --provider.cluster devnet

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Deploy to mainnet (requires funded keypair)
anchor deploy --provider.cluster mainnet-beta
```

### Verify Program ID

```bash
anchor keys list
# Should match CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh (devnet)
```

---

## Key Management

```bash
# Generate new keypair
solana-keygen new --outfile /secure/path/new-admin.json

# Backup existing keypair (store encrypted offsite)
cp ~/.config/solana/admin.json /secure-backup/admin-$(date +%Y%m%d).json

# Verify a keypair
solana-keygen pubkey /path/to/keypair.json
```

> **Production Recommendation:** Use [Squads Protocol](https://squads.so/) for multisig admin key management.

---

## Monitoring & Alerting

### Key Metrics

| Metric | Alert Threshold | Action |
|---|---|---|
| Protocol pause event | Any occurrence | Immediate escalation |
| Blacklist additions per day | > 10 | Compliance review |
| Seizure events | Any occurrence | Legal notification |
| Admin key usage | Any occurrence | Security audit |
| Failed transfer hook rate | > 5% | Investigate anomalies |

### Log Monitoring

```bash
# Watch live program logs
solana logs CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh

# Filter for compliance events in captured logs
grep -E "frozen|blacklist|seized|paused" program_logs.txt
```

---

## Contact Information

| Role | Contact |
|---|---|
| Emergency Hotline | +1-XXX-XXX-XXXX |
| Compliance Team | compliance@company.com |
| Technical Support | devops@company.com |
| Legal Department | legal@company.com |

---

## Changelog

| Version | Date | Changes |
|---|---|---|
| v1.0.0 | 2026-03-12 | Initial deployment |
| v1.1.0 | – | Enhanced monitoring |
| v1.2.0 | – | Emergency procedures added |