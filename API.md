# API Reference

This document provides the complete API reference for the Solana Stablecoin Standard — covering the on-chain program instructions, account schemas, error codes, the TypeScript SDK interface, and a backend API reference template.

---

## On-Chain Program API

**Program ID:** `CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh`  
**Framework:** Anchor  
**Token Standard:** Token-2022

---

## Instructions

### `initialize_stablecoin`

Initialize the stablecoin config account.

**Authority:** Any signer (becomes admin and minter)

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `decimals` | `u8` | Token decimal places (e.g., `6` for USDC-style) |

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | init (writable) | New `StablecoinConfig` account |
| `admin` | signer, writable | Payer and initial admin authority |
| `mint` | read | Token mint address |
| `treasury` | read | Treasury token account |
| `system_program` | read | System Program |

**Errors:** None (first-time init)

---

### `create_mint`

Signals that the mint account has been created (no-op instruction for IDL tracking).

**Authority:** Admin signer

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `admin` | signer, writable | Admin keypair |
| `mint` | writable | The token mint account |
| `system_program` | read | System Program |
| `token_program` | read | Token-2022 Program |

---

### `mint_tokens`

Mint new tokens to a recipient's token account.

**Authority:** `config.minter`

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `amount` | `u64` | Number of tokens to mint (base units) |

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `admin` | signer, writable | Must equal `config.minter` |
| `mint` | writable | Must equal `config.mint` |
| `user_token_account` | writable | Recipient token account |
| `token_program` | read | Token-2022 Program |

**Errors:**

| Error | Condition |
|---|---|
| `ProtocolPaused` | `config.paused == true` |
| `Unauthorized` | signer ≠ `config.minter` |
| `InvalidMint` | provided mint ≠ `config.mint` |

---

### `burn_tokens`

Burn tokens from the caller's token account.

**Authority:** Token account owner (user)

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `amount` | `u64` | Number of tokens to burn |

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `user` | signer, writable | Token account owner |
| `mint` | writable | Must equal `config.mint` |
| `user_token_account` | writable | Token account to burn from |
| `token_program` | read | Token-2022 Program |

**Errors:**

| Error | Condition |
|---|---|
| `ProtocolPaused` | `config.paused == true` |
| `InvalidMint` | provided mint ≠ `config.mint` |

---

### `freeze_account`

Freeze a user's token account. Frozen accounts cannot send or receive tokens.

**Authority:** `config.admin`

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `admin` | signer, writable | Must equal `config.admin` |
| `mint` | writable | Must equal `config.mint` |
| `user_token_account` | writable | Account to freeze |
| `token_program` | read | Token-2022 Program |

**Errors:**

| Error | Condition |
|---|---|
| `Unauthorized` | signer ≠ `config.admin` |
| `InvalidMint` | provided mint ≠ `config.mint` |

---

### `thaw_account`

Unfreeze a previously frozen token account.

**Authority:** `config.admin`

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `admin` | signer, writable | Must equal `config.admin` |
| `mint` | writable | Must equal `config.mint` |
| `user_token_account` | writable | Account to thaw |
| `token_program` | read | Token-2022 Program |

**Errors:** Same as `freeze_account`

---

### `pause`

Pause the protocol. Blocks all future `mint_tokens` calls.

**Authority:** `config.admin`

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `admin` | signer | Must equal `config.admin` |

**Errors:**

| Error | Condition |
|---|---|
| `Unauthorized` | signer ≠ `config.admin` |

---

### `unpause`

Unpause the protocol, resuming normal mint operations.

**Authority:** `config.admin`

**Accounts:** Same as `pause`

**Errors:** Same as `pause`

---

### `add_to_blacklist`

Create a Blacklist PDA for a wallet, blocking it from all transfers.

**Authority:** `config.admin`

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `wallet` | `Pubkey` | Wallet to blacklist |

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `blacklist` | init (writable) | New Blacklist PDA at `["blacklist", wallet]` |
| `admin` | signer, writable | Must equal `config.admin` |
| `system_program` | read | System Program |

**PDA Derivation:**
```
seeds = [b"blacklist", wallet.as_ref()]
program = STABLECOIN_PROGRAM_ID
```

**Errors:**

| Error | Condition |
|---|---|
| `Unauthorized` | signer ≠ `config.admin` |

---

### `remove_from_blacklist`

Close the Blacklist PDA for a wallet. Rent is returned to admin.

**Authority:** `config.admin`

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `blacklist` | writable (close → admin) | Blacklist PDA to close |
| `admin` | signer, writable | Must equal `config.admin` |

**Errors:**

| Error | Condition |
|---|---|
| `Unauthorized` | signer ≠ `config.admin` |

---

### `seize`

Forcibly transfer tokens from a user's account to the treasury.

**Authority:** `config.admin`

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `amount` | `u64` | Amount to seize |

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `admin` | signer, writable | Must equal `config.admin` |
| `mint` | writable | Must equal `config.mint` |
| `user_token_account` | writable | Source account |
| `treasury_token_account` | writable | Destination (treasury) |
| `token_program` | read | Token-2022 Program |

**Errors:**

| Error | Condition |
|---|---|
| `Unauthorized` | signer ≠ `config.admin` |
| `InvalidMint` | provided mint ≠ `config.mint` |

---

### `update_minter`

Change the authorized minter address.

**Authority:** `config.admin`

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `new_minter` | `Pubkey` | New minter public key |

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `admin` | signer | Must equal `config.admin` |

**Errors:**

| Error | Condition |
|---|---|
| `Unauthorized` | signer ≠ `config.admin` |

---

### `transfer_authority`

Transfer admin authority to a new public key.

**Authority:** `config.admin`

**Parameters:**

| Name | Type | Description |
|---|---|---|
| `new_admin` | `Pubkey` | New admin public key |

**Accounts:**

| Name | Mutability | Description |
|---|---|---|
| `config` | writable | `StablecoinConfig` account |
| `admin` | signer | Must equal current `config.admin` |

**Errors:**

| Error | Condition |
|---|---|
| `Unauthorized` | signer ≠ `config.admin` |

---

## Account Schemas

### `StablecoinConfig`

Account discriminator: 8 bytes (Anchor auto-generated)

| Field | Type | Size | Description |
|---|---|---|---|
| `admin` | `Pubkey` | 32 | Admin authority |
| `mint` | `Pubkey` | 32 | Token mint address |
| `treasury` | `Pubkey` | 32 | Treasury token account |
| `decimals` | `u8` | 1 | Token decimals |
| `paused` | `bool` | 1 | Protocol pause state |
| `minter` | `Pubkey` | 32 | Authorized minter |
| **Total** | | **138 bytes** | (8 discriminator + 130 data) |

### `Blacklist`

| Field | Type | Size | Description |
|---|---|---|---|
| `wallet` | `Pubkey` | 32 | Blacklisted wallet address |
| **Total** | | **40 bytes** | (8 discriminator + 32 data) |

---

## Error Codes

| Code | Anchor Name | HTTP-equivalent | Message |
|---|---|---|---|
| 6000 | `Unauthorized` | 403 | "Unauthorized" |
| 6001 | `ProtocolPaused` | 503 | "Protocol is paused" |
| 6002 | `InvalidMint` | 400 | "Invalid mint account" |
| 6003 | `Blacklisted` | 403 | "Wallet is blacklisted" |

---

## Transfer Hook Program API

**Program ID:** `C7TEPRE4dATYHHV3wafWybBD6z5FkngfAuBVNvezmBA4`

The Transfer Hook Program is invoked automatically by Token-2022 on every token transfer. It does not have a public-facing instruction API; it is called internally by the Token-2022 runtime.

### Behavior

1. Receives sender and receiver wallet addresses from Token-2022
2. Derives Blacklist PDA for each: `["blacklist", wallet]` against the Stablecoin Controller program ID
3. If either PDA exists → transaction fails with `Blacklisted` error
4. If neither PDA exists → transfer proceeds normally

---

## Backend API Reference (Off-Chain)

> The following endpoints represent a recommended off-chain backend service for monitoring, querying state, and submit transactions. This is a **reference template** — the actual implementation depends on your backend infrastructure.

### `GET /api/v1/config`

Returns the current stablecoin config.

**Response:**
```json
{
  "admin": "AdminPubkey...",
  "mint": "Henxk2RfJY2UqkiihsiT3fqHPLrwHnJ68ejfVVBtYBFN",
  "treasury": "TreasuryPubkey...",
  "decimals": 6,
  "paused": false,
  "minter": "MinterPubkey..."
}
```

### `GET /api/v1/supply`

Returns the current token supply.

**Response:**
```json
{
  "mint": "Henxk2RfJY2UqkiihsiT3fqHPLrwHnJ68ejfVVBtYBFN",
  "supply": "1000000000000",
  "decimals": 6,
  "displaySupply": "1000000.000000"
}
```

### `GET /api/v1/blacklist`

Returns all currently blacklisted wallets.

**Response:**
```json
{
  "blacklistedWallets": [
    "Wallet1Pubkey...",
    "Wallet2Pubkey..."
  ],
  "count": 2
}
```

### `GET /api/v1/blacklist/:wallet`

Check if a specific wallet is blacklisted.

**Response (blacklisted):**
```json
{ "wallet": "Wallet1Pubkey...", "blacklisted": true }
```

**Response (not blacklisted):**
```json
{ "wallet": "CleanWalletPubkey...", "blacklisted": false }
```

### `GET /api/v1/account/:tokenAccount`

Get token account info.

**Response:**
```json
{
  "address": "TokenAccountPubkey...",
  "owner": "OwnerWalletPubkey...",
  "balance": "500000000",
  "frozen": false,
  "mint": "Henxk2RfJY2UqkiihsiT3fqHPLrwHnJ68ejfVVBtYBFN"
}
```

### `GET /api/v1/events`

Returns recent compliance events (from indexed program logs).

**Query params:** `limit` (default: 20), `action` (optional filter), `since` (ISO timestamp)

**Response:**
```json
{
  "events": [
    {
      "timestamp": "2026-03-12T13:00:00Z",
      "slot": 312948293,
      "signature": "3RfJq...",
      "action": "ACCOUNT_FROZEN",
      "actor": "AdminPubkey...",
      "subject": "UserWalletPubkey..."
    }
  ]
}
```

---

## Rate Limits

| Endpoint | Limit |
|---|---|
| Read endpoints | 100 req/min |
| Write (mint, freeze, blacklist) | 10 req/min (admin only) |

---

## Related

- [SDK.md](SDK.md) — TypeScript SDK with code examples
- [OPERATIONS.md](OPERATIONS.md) — Step-by-step operational procedures
- [ARCHITECTURE.md](ARCHITECTURE.md) — System design and data flows
- [COMPLIANCE.md](COMPLIANCE.md) — Audit trail and regulatory notes
