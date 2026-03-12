# SSS-2: Compliant Stablecoin Standard

**Version 1.0.0 | March 2026**

> SSS-2 extends **SSS-1** with proactive compliance: **permanent delegate**, **transfer hook enforcement**, and **blacklist-gated transfers**. Every token movement is validated on-chain — gaps are not possible.

---

## Abstract

SSS-2 targets regulated stablecoin deployments — USDC/USDT-class tokens where regulators expect on-chain blacklist enforcement and seizure capabilities. The key additions over SSS-1:

- **Permanent Delegate** on the Token-2022 mint — gives the protocol authority to move tokens from any account without the owner's signature (required for `seize`)
- **Transfer Hook Program** — validates every transfer against the blacklist atomically; no client-side bypass possible
- **Blacklist PDAs** — O(1) lookup keys owned by the stablecoin program
- **Additional compliance roles** — blacklister, seizer — so the master authority isn't needed for day-to-day enforcement

---

## SSS-1 Compliance Required

All instructions and behavioral guarantees mandated by SSS-1 MUST be fully implemented. SSS-2 adds to them.

> Refer to [SSS-1.md](SSS-1.md) for the base specification.

---

## Initialization Difference

SSS-2 is activated at `initialize` time by setting feature flags:

```rust
// SSS-2 initialization parameters
StablecoinConfig {
    // ... SSS-1 fields ...
    enable_permanent_delegate: true,   // ← required for seize
    enable_transfer_hook: true,        // ← required for blacklist enforcement
    default_account_frozen: false,     // optional: true = allowlist model
}
```

The **same program** handles both presets. SSS-2-only instructions MUST fail gracefully with `FeatureNotEnabled` if called on an SSS-1-initialized config.

---

## Extended Role Model

| Role | Capabilities | Notes |
|---|---|---|
| **Master Authority** | All operations | Root of trust; should be multisig in production |
| **Minter** | `mint_tokens` (with per-minter quotas) | Can be rotated without full admin key |
| **Burner** | `burn_tokens` on own account | User-controlled |
| **Blacklister** | `add_to_blacklist`, `remove_from_blacklist` | Can be a compliance team key |
| **Pauser** | `pause`, `unpause` | Operational role for emergencies |
| **Seizer** | `seize` | Restricted role; should require multisig |

---

## Additional Account: Blacklist PDA

```rust
pub struct Blacklist {
    pub wallet: Pubkey, // Blacklisted wallet address
}
// PDA: seeds = [b"blacklist", wallet_pubkey]
// Size: 8 (discriminator) + 32 = 40 bytes
```

**Why PDA-based:** O(1) lookup during Transfer Hook execution; no iteration needed; deterministically verifiable; automatically closed (rent refunded) on removal.

---

## Permanent Delegate

Token-2022's Permanent Delegate extension gives a designated authority the ability to transfer or burn tokens from **any** token account, without the account owner's consent. In SSS-2:

- The permanent delegate is set to the stablecoin program's PDA or the admin's key at mint creation
- This is what makes `seize` possible — tokens can be moved to treasury even if the user does not cooperate
- The transfer hook still fires on seized transfers, but the permanent delegate overrides account ownership

> **Security note:** The permanent delegate is a significant privilege. In production, the delegate authority should be a multisig.

---

## Transfer Hook

The Transfer Hook Program is registered on the Token-2022 mint at creation time. It is called by the Token-2022 runtime on **every** `transfer` and `transferChecked` instruction.

### Hook Contract

```
Token-2022.transfer(amount)
    → ExtraAccountMetaList lookup
    → Transfer Hook Program invoked
    → derives ["blacklist", sender] PDA → check exists
    → derives ["blacklist", receiver] PDA → check exists
    → if either exists: reject (tx reverts atomically)
    → if neither: approve → transfer completes
```

Key properties:
- **Cannot be bypassed** by client code — the hook runs at the protocol level
- **Atomic** — hook rejection reverts the entire transaction including the transfer
- **Stateless per call** — reads PDA existence; no additional writes during transfer

---

## SSS-2 Additional Instructions

### `add_to_blacklist(wallet: Pubkey)`

- **Authority:** Blacklister role (or admin)
- **Effect:** Creates Blacklist PDA; wallet is immediately blocked from all transfers
- **Constraints:**
  - MUST fail with `FeatureNotEnabled` if `enable_transfer_hook == false`
  - MUST fail with `Unauthorized` if signer is not blacklister or admin
  - PDA constraint prevents duplicate entries

### `remove_from_blacklist()`

- **Authority:** Blacklister role (or admin)
- **Effect:** Closes Blacklist PDA; rent returned to admin; wallet transfers resume
- **Constraints:** Same as `add_to_blacklist`

### `seize(amount: u64)`

- **Authority:** Seizer role (or admin)
- **Effect:** Uses the **permanent delegate** to transfer `amount` tokens from target account to treasury — no user consent required
- **Constraints:**
  - MUST fail with `FeatureNotEnabled` if `enable_permanent_delegate == false`
  - MUST fail with `Unauthorized` if signer is not seizer or admin
  - MUST fail with `InvalidMint` if mint mismatch
  - Best practice: target account should be frozen before seizure

---

## Additional Error Code

| Code | Message | Added By |
|---|---|---|
| `Blacklisted` | "Wallet is blacklisted" | SSS-2 |
| `FeatureNotEnabled` | "Feature not enabled for this preset" | SSS-1 (used by all) |

---

## Compliance Properties

| Property | Mechanism |
|---|---|
| **Transfer enforcement** | Transfer Hook fires on every transfer; no client bypass |
| **Blacklist atomicity** | Rejection reverts entire transaction |
| **Seizure without consent** | Permanent delegate removes need for user signature |
| **Freeze coverage** | SSS-1 `freeze_account` + SSS-2 `default_account_frozen` allowlist model |
| **Audit trail** | All compliance actions emit on-chain logs; indexable off-chain |

---

## Conformance Checklist

An implementation is SSS-2 conformant if:

- [ ] All SSS-1 requirements are met
- [ ] Token-2022 mint created with **Permanent Delegate** extension enabled
- [ ] Transfer Hook Program registered on the mint
- [ ] `add_to_blacklist` / `remove_from_blacklist` with PDA seeds `["blacklist", wallet]`
- [ ] `seize` uses permanent delegate (not admin-signed user transfer)
- [ ] Extended role model implemented (blacklister, seizer distinct from master authority)
- [ ] SSS-2 instructions fail with `FeatureNotEnabled` on SSS-1-initialized config
- [ ] `Blacklisted` and `FeatureNotEnabled` error codes present

---

## CLI (SSS-2 preset)

```bash
sss-token init --preset sss-2 --name "My Regulated USD" --symbol "RUSD" --decimals 6

# Compliance operations
sss-token blacklist add <address> --reason "OFAC match"
sss-token blacklist remove <address>
sss-token seize <address> --to <treasury>
sss-token freeze <address>
sss-token thaw <address>

# Monitoring
sss-token audit-log --action BLACKLIST_ADD
sss-token holders --min-balance 1000
```

---

## Reference Implementation

See `programs/my-solana-project/src/lib.rs` (main program) and `programs/transfer_hook/src/lib.rs` (hook program).

**TypeScript:**
```typescript
import { SolanaStablecoin, Presets } from "@stbr/sss-token";

const stable = await SolanaStablecoin.create(connection, {
  preset: Presets.SSS_2,
  name: "My Regulated USD",
  symbol: "RUSD",
  decimals: 6,
  authority: adminKeypair,
});

await stable.compliance.blacklistAdd(suspiciousWallet, "Sanctions match");
await stable.freeze(userTokenAccount);
await stable.compliance.seize(frozenAccount, treasury);
```
