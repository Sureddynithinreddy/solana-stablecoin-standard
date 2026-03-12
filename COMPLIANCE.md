# Compliance Guide

This document covers regulatory considerations, compliance mechanisms, and audit trail formats for the Solana Stablecoin Standard implementation.

---

## Overview

The Solana Stablecoin Standard (SSS-2) is designed to meet the compliance requirements commonly imposed on digital stablecoin issuers, including:

- **OFAC Sanctions Screening** — via wallet blacklisting and Transfer Hook enforcement
- **AML / KYC Monitoring** — via on-chain event logging and audit trails
- **Account Remediation** — via account freezing, thawing, and token seizure
- **Emergency Controls** — via protocol-wide pause functionality

---

## Regulatory Considerations

### Who Should Read This

- **Compliance Officers**: Understand the on-chain controls available and their limitations.
- **Legal Counsel**: Evaluate the enforceability of the smart contract mechanisms.
- **Auditors**: Use the audit trail format below for transaction review.
- **Integrators**: Understand the compliance implications of deploying or integrating this stablecoin.

### Key Regulatory Mechanisms

#### 1. Blacklisting (OFAC / Sanctions)

The blacklist prevents a sanctioned wallet from being involved in **any** token transfer:

- The blacklist entry is a PDA stored on-chain at `seeds = ["blacklist", wallet]`.
- The **Transfer Hook Program** checks this PDA on every token movement.
- Even if a user calls the token transfer instruction directly (bypassing the SDK), the hook executes at the protocol level and rejects the transaction.
- **Limitation**: The blacklist covers on-chain transfers only. Off-chain fiat or cross-chain movements are not affected.

#### 2. Account Freezing

Token accounts can be individually frozen by the admin:

- A frozen account cannot send or receive tokens (enforced by SPL Token-2022).
- This is useful for targeted compliance actions while allowing the protocol to continue operating.
- **Limitation**: Freezing requires the admin's private key. Consider a multisig admin for production.

#### 3. Token Seizure

Tokens can be forcibly transferred from a user's account to the treasury:

- Requires admin authority.
- Should only be executed following a court order, regulatory directive, or verified fraud case.
- Every seizure MUST be documented in the off-chain audit log.

#### 4. Protocol Pause

The admin can pause all minting operations across the entire protocol:

- Useful during security incidents, system audits, or regulatory investigations.
- Burns are NOT blocked by the pause (users can still reduce their exposure).
- **Limitation**: Only minting is paused; existing token transfers are not halted by pause alone. Combine with account freezing for stronger controls.

---

## Admin Key Management

| Risk Level | Recommended Control |
|---|---|
| **Development / Testnet** | Single admin keypair |
| **Staging** | 2-of-3 multisig (e.g., Squads Protocol) |
| **Production** | 3-of-5 multisig, HSM-backed |

Admin key operations that are logged on-chain:
- Authority transfer (`transfer_authority`)
- Minter update (`update_minter`)
- Any freeze / thaw / blacklist / seize action

---

## Audit Trail Format

All compliance-relevant on-chain events are emitted as Anchor `msg!` logs. Off-chain monitoring systems SHOULD capture and index these logs.

### Canonical Audit Event Format

Each log entry should be captured in the following JSON structure by an off-chain listener:

```json
{
  "timestamp": "2026-03-12T13:00:00Z",
  "slot": 312948293,
  "signature": "3RfJq...",
  "program": "CmEzDTfuEvkBxcki6mNEYZUHKAjYNkiK7rk6Wmcgx4Vh",
  "action": "ACCOUNT_FROZEN",
  "actor": "AdminWalletPubkey...",
  "subject": "UserWalletPubkey...",
  "amount": null,
  "reason": "Regulatory request",
  "authorized_by": "ComplianceTeamMember",
  "approved_by": "LegalCounsel"
}
```

### Action Types

| Action | Instruction | Severity |
|---|---|---|
| `STABLECOIN_INITIALIZED` | `initialize_stablecoin` | INFO |
| `TOKENS_MINTED` | `mint_tokens` | INFO |
| `TOKENS_BURNED` | `burn_tokens` | INFO |
| `PROTOCOL_PAUSED` | `pause` | HIGH |
| `PROTOCOL_UNPAUSED` | `unpause` | HIGH |
| `ACCOUNT_FROZEN` | `freeze_account` | HIGH |
| `ACCOUNT_THAWED` | `thaw_account` | MEDIUM |
| `WALLET_BLACKLISTED` | `add_to_blacklist` | HIGH |
| `WALLET_UNBLACKLISTED` | `remove_from_blacklist` | MEDIUM |
| `TOKENS_SEIZED` | `seize` | CRITICAL |
| `MINTER_UPDATED` | `update_minter` | HIGH |
| `ADMIN_TRANSFERRED` | `transfer_authority` | CRITICAL |
| `TRANSFER_BLOCKED` | (Transfer Hook rejection) | HIGH |

### On-Chain Log Messages (from `msg!`)

| Event | Log Message |
|---|---|
| Initialize | `"Stablecoin initialized"` |
| Mint | `"Tokens minted successfully"` |
| Burn | `"Tokens burned successfully"` |
| Freeze | `"Account frozen successfully"` |
| Thaw | `"Account unfrozen successfully"` |
| Pause | `"Protocol paused"` |
| Unpause | `"Protocol unpaused"` |
| Blacklist add | `"Wallet added to blacklist"` |
| Blacklist remove | `"Wallet removed from blacklist"` |
| Seize | `"Tokens seized to treasury"` |
| Minter update | `"Minter updated"` |
| Admin transfer | `"Admin authority transferred"` |

---

## Off-Chain Audit System (Recommended Stack)

```
Solana Validator Logs
        ↓
Log Streaming (e.g., Helius, QuickNode WebSocket)
        ↓
Event Indexer (Node.js / Python service)
        ↓
Audit Database (PostgreSQL / Elasticsearch)
        ↓
Compliance Dashboard / Report Generator
        ↓
Regulatory Reports (FinCEN, OFAC, etc.)
```

### Minimum Retention Requirements

| Record Type | Recommended Retention |
|---|---|
| All mint / burn events | 7 years |
| Freeze / thaw events | 7 years |
| Blacklist add / remove | 10 years |
| Seizure records | Indefinite (or per court order) |
| Admin key changes | Indefinite |

---

## Compliance Incident Response

### Severity Classification

| Level | Example | Response Time |
|---|---|---|
| **CRITICAL** | Token seizure, admin transfer, unexpected pause | Immediate (< 1 hour) |
| **HIGH** | Account freeze, new blacklist entry | Same business day |
| **MEDIUM** | Account thaw, blacklist removal | 48 hours |
| **INFO** | Routine mint / burn | Daily review |

### Response Checklist

1. **Detect**: Monitoring system alerts on HIGH/CRITICAL log event
2. **Assess**: Identify the action, actor, and subject wallet from the on-chain signature
3. **Document**: Create a compliance incident report with transaction signature and reason
4. **Escalate**: Notify compliance officer and legal counsel for CRITICAL events
5. **Archive**: Store the incident report in the audit database with the canonical format above

---

## Limitations & Disclaimer

> [!CAUTION]
> This smart contract system provides **technical mechanisms** for compliance enforcement. It does NOT constitute legal compliance in and of itself. Operators MUST:
> - Perform off-chain KYC/AML screening before onboarding users
> - Maintain legal authorizations for all seizure and blacklist actions
> - Consult legal counsel for jurisdiction-specific requirements
> - Ensure admin key management meets applicable regulatory standards

---

## Related Documents

- [SSS-1.md](SSS-1.md) — Minimal stablecoin standard
- [SSS-2.md](SSS-2.md) — Compliant stablecoin standard
- [OPERATIONS.md](OPERATIONS.md) — Operational procedures
- [ARCHITECTURE.md](ARCHITECTURE.md) — Technical architecture
