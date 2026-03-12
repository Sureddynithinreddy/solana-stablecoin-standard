# SSS-1: Minimal Stablecoin Standard

**Version 1.0.0 | March 2026**

> SSS-1 defines the **minimum viable on-chain interface** that a Solana-based stablecoin must implement to be standards-compliant. It provides mint authority, freeze authority, metadata, and core lifecycle management — everything a functional stablecoin needs and nothing more.

---

## Abstract

The Minimal Stablecoin Standard (SSS-1) specifies the core instructions, account structures, and behavioral guarantees for an SSS-compliant token. SSS-1 is suitable for simple stablecoins — internal tokens, DAO treasuries, ecosystem settlement. Compliance is **reactive**: operators freeze accounts as needed. SSS-2 extends SSS-1 with proactive transfer enforcement.

Any SSS-2 implementation is also SSS-1 compliant.

---

## Use Case

SSS-1 is appropriate when:
- You need a functional stablecoin without mandatory on-chain blacklist enforcement
- Compliance actions happen reactively (freeze an account after a problem is identified)
- You're building an internal token, DAO treasury token, or ecosystem settlement token
- You want minimal complexity with full lifecycle management

---

## Specification

### Required Token Properties

| Property | Requirement |
|---|---|
| Token Standard | Token-2022 (SPL Token-2022 program) |
| Mint Authority | Stored in config as `minter` |
| Freeze Authority | Required — admin must be the freeze authority on the mint |
| Metadata | Required — name, symbol, URI stored on-chain at init |
| Decimals | Set at initialization; MUST NOT change after deployment |

### Required Config Account: `StablecoinConfig`

```rust
pub struct StablecoinConfig {
    pub name: String,                    // Token name (e.g., "My USD")
    pub symbol: String,                  // Token symbol (e.g., "MYUSD")
    pub uri: String,                     // Metadata URI
    pub decimals: u8,                    // Token decimal precision
    // SSS-1 core
    pub admin: Pubkey,                   // Master authority
    pub mint: Pubkey,                    // Associated token mint
    pub treasury: Pubkey,                // Protocol treasury account
    pub paused: bool,                    // Protocol pause state
    pub minter: Pubkey,                  // Authorized minting role
    // --- Fields below are SSS-2 only, set to false/None for SSS-1 ---
    pub enable_permanent_delegate: bool, // false for SSS-1
    pub enable_transfer_hook: bool,      // false for SSS-1
    pub default_account_frozen: bool,    // false for SSS-1
}
```

> **Design note:** Using a single config account with feature flags allows the same program to support both presets via initialization parameters.

### Role Model (SSS-1)

| Role | Key Stored In | Capabilities |
|---|---|---|
| **Master Authority** | `config.admin` | All admin operations; the root of trust |
| **Minter** | `config.minter` | Can call `mint_tokens` (with per-minter quotas if implemented) |
| **Burner** | Token account owner | Can call `burn_tokens` on own account |
| **Pauser** | `config.admin` | Can pause/unpause (may be delegated in extended implementations) |

---

## Required Instructions

### `initialize(name, symbol, uri, decimals)`

- **Authority:** Any signer (becomes master authority)
- **Effect:** Creates `StablecoinConfig`; sets `admin = signer`, `minter = signer`, `paused = false`, `enable_permanent_delegate = false`, `enable_transfer_hook = false`
- **Constraints:** Decimals MUST NOT be zero

### `mint_tokens(amount: u64)`

- **Authority:** `minter`
- **Effect:** Mints `amount` tokens to the target token account via Token-2022 CPI
- **Constraints:**
  - MUST fail with `ProtocolPaused` if `config.paused == true`
  - MUST fail with `Unauthorized` if signer ≠ `config.minter`
  - MUST fail with `InvalidMint` if provided mint ≠ `config.mint`

### `burn_tokens(amount: u64)`

- **Authority:** Token account owner (user)
- **Effect:** Burns tokens from caller's token account
- **Constraints:**
  - MUST fail with `ProtocolPaused` if `config.paused == true`
  - MUST fail with `InvalidMint` if provided mint ≠ `config.mint`

### `freeze_account()`

- **Authority:** Master authority (`admin`)
- **Effect:** Freezes the target token account via Token-2022 CPI (account cannot send or receive)
- **Constraints:** MUST fail with `Unauthorized` if signer ≠ `config.admin`; MUST fail with `InvalidMint` if mint mismatch
- **Note:** This is a **SSS-1 core instruction** — reactive compliance

### `thaw_account()`

- **Authority:** Master authority (`admin`)
- **Effect:** Unfreezes a previously frozen token account
- **Constraints:** Same as `freeze_account`

### `pause()`

- **Authority:** `admin` (or designated pauser role)
- **Effect:** Sets `config.paused = true`; blocks all future minting

### `unpause()`

- **Authority:** `admin`
- **Effect:** Sets `config.paused = false`; resumes minting

### `update_minter(new_minter: Pubkey)`

- **Authority:** `admin`
- **Effect:** Updates `config.minter`
- **Note:** Setting to system program (`111...112`) effectively halts minting

### `update_roles(role, new_authority: Pubkey)`

- **Authority:** `admin`
- **Effect:** Updates a named role's authority (pauser, burner, etc.)

### `transfer_authority(new_admin: Pubkey)`

- **Authority:** Current `admin`
- **Effect:** Updates `config.admin` — caller immediately loses admin rights

---

## Required Error Codes

| Code | Message |
|---|---|
| `Unauthorized` | "Unauthorized" |
| `ProtocolPaused` | "Protocol is paused" |
| `InvalidMint` | "Invalid mint account" |
| `FeatureNotEnabled` | "Feature not enabled for this preset" |

---

## What SSS-1 Does NOT Require

- Wallet-level blacklisting (proactive transfer blocking)
- Permanent delegate (forced token movements)
- Transfer Hook (on-transfer compliance checks)
- Token seizure via permanent delegate

These are defined in **SSS-2**.

---

## Conformance

An implementation is SSS-1 conformant if:

- [ ] Single configurable program supporting SSS-1 via initialization parameters
- [ ] `StablecoinConfig` contains all required fields including feature flags
- [ ] `freeze_account` and `thaw_account` implemented as core instructions
- [ ] All required instructions implemented with correct authority checks
- [ ] `enable_permanent_delegate` and `enable_transfer_hook` default to `false`
- [ ] SSS-2 instructions fail gracefully with `FeatureNotEnabled` if called on SSS-1 init

---

## Reference Implementation

See `programs/my-solana-project/src/lib.rs` and [SDK.md](SDK.md) for TypeScript usage.

**CLI:**
```bash
sss-token init --preset sss-1 --name "My USD" --symbol "MYUSD" --decimals 6
sss-token mint <recipient> <amount>
sss-token freeze <address>
sss-token pause
sss-token status
```
