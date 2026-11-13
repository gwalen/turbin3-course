# Turbin3 Q4 2026

Global monorepo for Turbin3 Q4 2026 Builders Cohort course work. Each task lives in its own nested project.

- `anchor-v1`: same project, created with Anchor v1
- `anchor-v2`: same project, created with Anchor v2

## Prerequisite vault 

### Anchor v1

Project path: [`anchor-v1/pre-req-vault`](anchor-v1/pre-req-vault)

Run Anchor commands from that directory, not the repository root:

```bash
cd anchor-v1/pre-req-vault
anchor build
```

### Anchor v2

TBD

## Vault

### Anchor v1

Project path: [`anchor-v1/vault`](anchor-v1/vault)

Each user has a personal vault. They can deposit SOL, withdraw it, and close the vault.

Run commands from that directory:

```bash
cd anchor-v1/vault
just build
just rust
```

## Token-swap escrow

Project path: [`anchor-v1/escrow`](anchor-v1/escrow). See its [README](anchor-v1/escrow/README.md) for account layout, instruction flows, and timed expiry rules.

The maker locks token A and asks for a fixed amount of token B. A taker pays B to receive A; the maker can refund an untaken basic offer. Timed offers lock refunds until expiry and reject takes from the expiry timestamp onward. Both paths return escrow and vault rent to the maker.

Build and run the Rust LiteSVM tests from the project directory:

```bash
cd anchor-v1/escrow
just build
just rust
```

The project uses Anchor 1.2.1, Rust 1.97.1, and LiteSVM 0.18.2 with Anchor's default sBPF v3 build. `just rust-logs` prints transaction logs.
