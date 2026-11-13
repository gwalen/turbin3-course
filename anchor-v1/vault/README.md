# Vault

A personal SOL vault. A user deposits SOL, withdraws it, and closes the vault to get everything back.

Each vault is a PDA derived from the user's wallet, so only that user can use it.

## Instructions

| Instruction | What it does |
|---|---|
| `initialize` | Creates the user's vault. |
| `deposit(amount)` | Moves SOL from the user into the vault. The minimum is 0.01 SOL. |
| `withdraw(amount)` | Moves SOL back to the user. Afterwards the vault holds 0 or at least 0.01 SOL. |
| `close` | Sends all SOL back to the user and closes the vault. |

The 0.01 SOL minimum keeps the vault above the rent-exempt minimum, so a transfer never fails with `InsufficientFundsForRent`.

## Build and test

You need [just](https://just.systems/), Anchor CLI 1.2.1 and Rust 1.97.1.

```console
just build      # build the program
just rust       # run the LiteSVM tests
just rust-logs  # run the tests and print program logs
```
