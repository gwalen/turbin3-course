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
