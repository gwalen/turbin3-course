# Vault

## How it works

Each user has a personal vault made of two PDAs.

| Account | Seeds | Owner | Holds |
|---|---|---|---|
| `vault_state` | `["state", user]` | vault program | `vault_bump`, `state_bump` |
| `vault` | `["vault", vault_state]` | System Program | deposited lamports |

`initialize` creates `vault_state` and stores both bumps. The `vault` account is a system account with no data. It comes into existence on the first deposit, when lamports are transferred into it.

`deposit` moves SOL from the user to the vault. `withdraw` moves SOL back. The vault PDA signs that transfer. `close` sends the vault's remaining lamports to the user and returns the `vault_state` rent.

`user` is a seed of `vault_state`, so a different signer derives a different address. A stranger who passes someone else's vault fails the seeds constraint.

### Minimum balance

The vault balance is always 0 or at least 0.01 SOL (`VaultState::MIN_BALANCE`, 10,000,000 lamports).

A system account with no data must hold either 0 lamports or at least the rent-exempt minimum for 0 bytes, which is 890,880 lamports. A balance between those two values fails in the runtime with `InsufficientFundsForRent`. 0.01 SOL stays above that floor, so the program does not read the Rent sysvar.

- A deposit is at least 0.01 SOL.
- After a withdrawal, the vault holds exactly 0 or at least 0.01 SOL.

## Build and test

This project uses [just](https://just.systems/). Install it with Homebrew or Cargo.

```console
brew install just
```

```console
cargo install just
```

You also need Anchor 1.2.2 and Rust 1.97.1.

Build the program:

```console
just build
```

Tests are Rust integration tests that run in-process against [LiteSVM](https://github.com/LiteSVM/litesvm) 0.18.2:

```console
just rust
```

To print program logs:

```console
just rust-logs
```
