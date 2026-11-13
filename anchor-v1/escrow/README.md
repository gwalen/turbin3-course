# Escrow

A token-swap escrow. The maker locks token A and asks for a fixed amount of token B. A taker pays that amount of B and receives all of A. Until someone takes the offer, the maker can refund it.

Each offer is a PDA derived from the maker and a `seed`, so one maker can have several offers open. Token A sits in a vault, which is a token account owned by that PDA. Only the program can move tokens out of it.

## Instructions

| Instruction | What it does |
|---|---|
| `make(seed, deposit, receive)` | Creates an offer and moves `deposit` of token A into the vault. |
| `take` | The taker sends `receive` of token B to the maker and gets the vault's token A. |
| `refund` | The maker gets token A back. |
| `make_timed(seed, deposit, receive, expires_at)` | Creates an offer that expires at `expires_at`, a Unix timestamp in seconds. |
| `take_expired` | Takes a timed offer. Works only **before** `expires_at`. |
| `refund_expired` | The maker gets token A back. Works only from `expires_at` onward. |

A timed offer is locked until it expires: the maker can't refund it while a taker can still take it. The refund isn't automatic, so the maker has to call `refund_expired`.

Every take and refund closes the offer and the vault and returns their rent to the maker.

The escrow works with SPL Token and Token-2022. It rejects mints with a transfer fee, a permanent delegate or a transfer hook.

## Build and test

You need [just](https://just.systems/), Anchor CLI 1.2.1 and Rust 1.97.1.

```console
just build      # build the program
just rust       # run the LiteSVM tests
just rust-logs  # run the tests and print program logs
```
