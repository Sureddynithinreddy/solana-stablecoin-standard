# SDK Reference

The TypeScript SDK (`@stbr/sss-token`) provides a developer-friendly, preset-driven interface to the Solana Stablecoin Standard programs. It wraps the Anchor client and exposes all on-chain instructions as typed async methods.

---

## Installation

```bash
cd sdk
npm install
# or: npm install @stbr/sss-token
```

**Dependencies:**
- `@coral-xyz/anchor` - Anchor framework client
- `@solana/web3.js` - Solana Web3 primitives

---

## Admin CLI

The `sss-token` CLI is a critical operator tool for executing actions fast without writing TypeScript.

### Installation

```bash
npm install -g @stbr/sss-token
# or: npx @stbr/sss-token
```

### Initialization

```bash
# Minimal stablecoin (SSS-1) - no compliance module
sss-token init --preset sss-1 --name "My USD" --symbol "MYUSD" --decimals 6

# Compliant stablecoin (SSS-2) - transfer hook + blacklist + permanent delegate
sss-token init --preset sss-2 --name "Regulated USD" --symbol "RUSD" --decimals 6

# Fully custom configuration via TOML file
sss-token init --custom config.toml
```

**`config.toml` example:**
```toml
name = "Custom Stable"
symbol = "CUSD"
decimals = 6
enable_permanent_delegate = true
enable_transfer_hook = false
default_account_frozen = false
```

### Core Operations (all presets)

```bash
sss-token mint <recipient> <amount>       # Mint to recipient token account
sss-token burn <amount>                   # Burn from caller's account
sss-token freeze <address>                # Freeze a token account
sss-token thaw <address>                  # Thaw a frozen account
sss-token pause                           # Pause minting protocol-wide
sss-token unpause                         # Resume minting
sss-token status                          # Show config, paused state, minter
sss-token supply                          # Show current token supply
```

### SSS-2 Compliance Commands

```bash
# Blacklist management
sss-token blacklist add <address> --reason "OFAC match"
sss-token blacklist remove <address>

# Seizure (uses permanent delegate — no user consent required)
sss-token seize <address> --to <treasury>

# Audit log
sss-token audit-log                      # All compliance events
sss-token audit-log --action BLACKLIST_ADD
sss-token audit-log --since 2026-03-01
```

### Role Management

```bash
sss-token minters list                    # List all minters
sss-token minters add <address>           # Add minter (admin only)
sss-token minters remove <address>        # Remove minter
sss-token holders                         # List all token holders
sss-token holders --min-balance 1000      # Filter by minimum balance
```

---


## Preset Configurations

The SDK exposes a `SolanaStablecoin.create()` factory with preset constants and custom extension config:

```typescript
import { SolanaStablecoin, Presets } from "@stbr/sss-token";
import { Connection } from "@solana/web3.js";

const connection = new Connection("https://api.devnet.solana.com");

// --- SSS-1: Minimal stablecoin (mint authority + freeze authority + metadata) ---
const stableSSS1 = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_1,
  name: "My USD",
  symbol: "MYUSD",
  decimals: 6,
  authority: adminKeypair,
});

await stableSSS1.mint({ recipient: userTokenAccount, amount: 1_000_000, minter: adminKeypair });
await stableSSS1.freeze(userTokenAccount);  // reactive compliance
const supply = await stableSSS1.getTotalSupply();

// --- SSS-2: Compliant stablecoin (+ permanent delegate + transfer hook + blacklist) ---
const stableSSS2 = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_2,
  name: "Regulated USD",
  symbol: "RUSD",
  decimals: 6,
  authority: adminKeypair,
});

await stableSSS2.compliance.blacklistAdd(suspiciousWallet, "Sanctions match");
await stableSSS2.freeze(userTokenAccount);
await stableSSS2.compliance.seize(frozenAccount, treasury);  // uses permanent delegate

// --- Custom config: pick exactly the extensions you need ---
const custom = await SolanaStablecoin.create(connection, {
  name: "Custom Stable",
  symbol: "CUSD",
  decimals: 6,
  extensions: {
    permanentDelegate: true,
    transferHook: false,         // No on-transfer enforcement
    defaultAccountFrozen: false,
  },
  authority: adminKeypair,
});
```

### What Each Preset Sets

| `StablecoinConfig` field | SSS-1 | SSS-2 |
|---|:---:|:---:|
| `enable_permanent_delegate` | `false` | `true` |
| `enable_transfer_hook` | `false` | `true` |
| `default_account_frozen` | `false` | Optional |

> **SSS-2 instructions called on an SSS-1 config will fail with `FeatureNotEnabled`** — feature gating is enforced on-chain.

---

## Custom Configuration

To point the SDK at a custom program ID or mint:

```typescript
import { StablecoinSDK } from "./sdk";
import { AnchorProvider } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

// Override the default program ID via environment variable or direct override
process.env.STABLECOIN_PROGRAM_ID = "YourCustomProgramId...";

const provider = AnchorProvider.env();
const sdk = new StablecoinSDK(provider);
```

Or by modifying `sdk/constants.ts`:

```typescript
// sdk/constants.ts
import { PublicKey } from "@solana/web3.js";

export const STABLECOIN_PROGRAM_ID = new PublicKey(
  "CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh" // Replace for custom deployment
);
```

---

## TypeScript Examples

### Initialize a New Stablecoin

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Keypair, SystemProgram } from "@solana/web3.js";
import { StablecoinSDK } from "./sdk";

const provider = anchor.AnchorProvider.env();
const sdk = new StablecoinSDK(provider);

const configAccount = Keypair.generate();
const mint = new PublicKey("Henxk2RfJY2UqkiihsiT3fqHPLrwHnJ68ejfVVBtYBFN");
const treasury = new PublicKey("YourTreasuryTokenAccount...");

await sdk.program.methods
  .initializeStablecoin(6) // 6 decimals
  .accounts({
    config: configAccount.publicKey,
    admin: provider.wallet.publicKey,
    mint,
    treasury,
    systemProgram: SystemProgram.programId,
  })
  .signers([configAccount])
  .rpc();

console.log("Stablecoin initialized! Config:", configAccount.publicKey.toString());
```

### Mint Tokens to a User

```typescript
import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import { StablecoinSDK } from "./sdk";

const provider = anchor.AnchorProvider.env();
const sdk = new StablecoinSDK(provider);

const admin = provider.wallet.publicKey;
const mint = new PublicKey("Henxk2RfJY2UqkiihsiT3fqHPLrwHnJ68ejfVVBtYBFN");
const userTokenAccount = new PublicKey("UserTokenAccountAddress...");

const txSig = await sdk.mintTokens(admin, mint, userTokenAccount, 1_000_000);
console.log("Minted! Transaction:", txSig);
```

### Burn Tokens

```typescript
await sdk.program.methods
  .burnTokens(new anchor.BN(500_000))
  .accounts({
    config: configPubkey,
    user: userKeypair.publicKey,
    mint,
    userTokenAccount,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .signers([userKeypair])
  .rpc();
```

### Freeze a User's Token Account

```typescript
await sdk.program.methods
  .freezeAccount()
  .accounts({
    config: configPubkey,
    admin: adminKeypair.publicKey,
    mint,
    userTokenAccount: targetTokenAccount,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .signers([adminKeypair])
  .rpc();

console.log("Account frozen:", targetTokenAccount.toString());
```

### Thaw a Frozen Account

```typescript
await sdk.program.methods
  .thawAccount()
  .accounts({
    config: configPubkey,
    admin: adminKeypair.publicKey,
    mint,
    userTokenAccount: targetTokenAccount,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .signers([adminKeypair])
  .rpc();
```

### Pause and Unpause Protocol

```typescript
// Pause all minting
await sdk.program.methods
  .pause()
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();

// Later, resume
await sdk.program.methods
  .unpause()
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();
```

### Add / Remove Wallet from Blacklist

```typescript
import { PublicKey } from "@solana/web3.js";

const suspiciousWallet = new PublicKey("WalletToBlacklist...");

// Compute PDA
const [blacklistPDA] = PublicKey.findProgramAddressSync(
  [Buffer.from("blacklist"), suspiciousWallet.toBuffer()],
  STABLECOIN_PROGRAM_ID
);

// Add to blacklist
await sdk.program.methods
  .addToBlacklist(suspiciousWallet)
  .accounts({
    config: configPubkey,
    blacklist: blacklistPDA,
    admin: adminKeypair.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .signers([adminKeypair])
  .rpc();

// Remove from blacklist (closes PDA, refunds rent)
await sdk.program.methods
  .removeFromBlacklist()
  .accounts({
    config: configPubkey,
    blacklist: blacklistPDA,
    admin: adminKeypair.publicKey,
  })
  .signers([adminKeypair])
  .rpc();
```

### Seize Tokens to Treasury

```typescript
const amount = new anchor.BN(1_000_000); // seize 1 USDS

await sdk.program.methods
  .seize(amount)
  .accounts({
    config: configPubkey,
    admin: adminKeypair.publicKey,
    mint,
    userTokenAccount: targetTokenAccount,
    treasuryTokenAccount: treasuryAccount,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .signers([adminKeypair])
  .rpc();
```

### Update Minter

```typescript
const newMinter = Keypair.generate();

await sdk.program.methods
  .updateMinter(newMinter.publicKey)
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();
```

### Transfer Admin Authority

```typescript
const newAdmin = Keypair.generate();

await sdk.program.methods
  .transferAuthority(newAdmin.publicKey)
  .accounts({ config: configPubkey, admin: adminKeypair.publicKey })
  .signers([adminKeypair])
  .rpc();
```

---

## SDK Class Reference

### `StablecoinSDK`

```typescript
class StablecoinSDK {
  program: Program;  // Anchor Program instance

  constructor(provider: AnchorProvider);

  // Convenience wrapper for mintTokens
  async mintTokens(
    admin: PublicKey,
    mint: PublicKey,
    userTokenAccount: PublicKey,
    amount: number
  ): Promise<string>; // Returns transaction signature
}
```

### Constants (`sdk/constants.ts`)

```typescript
export const STABLECOIN_PROGRAM_ID: PublicKey;
// = "CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh"
```

---

## Error Reference

| Error | Code | Description |
|---|---|---|
| `Unauthorized` | 6000 | Signer is not the configured admin or minter |
| `ProtocolPaused` | 6001 | Mint attempted while paused |
| `InvalidMint` | 6002 | Mint account doesn't match config |
| `Blacklisted` | 6003 | Wallet is on the blacklist |

---

## IDL

The full Anchor IDL is located at [`sdk/my_solana_project.json`](sdk/my_solana_project.json). It describes all instructions, accounts, and types for use with any Anchor-compatible client (TypeScript, Rust, Python via `anchorpy`).
