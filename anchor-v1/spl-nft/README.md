# spl-nft

This repo covers minting and transferring SPL tokens, plus minting, updating, transferring, and burning Metaplex Core NFTs.

If you look at `src/lib.rs`, the Anchor program is completely empty. Everything happens in `tests/spl_and_nft.rs`.

## Why there's no custom program

Writing a custom Solana program just to mint, transfer, or burn a token is pointless boilerplate. Your program would take the exact same accounts, check the exact same signatures, and immediately CPI into SPL Token or Metaplex Core anyway.

You only write a custom program when a PDA needs to own assets or enforce custom rules—like an escrow, a staking vault, or a gated mint. If a regular wallet is the authority, you just talk to `spl-token` and `mpl-core` directly from the client.

## Why the tests are the whole project

Since the on-chain programs already exist, the real work is building the right instructions and verifying how the accounts actually change.

Running these flows in `LiteSVM` lets us execute against the real SPL and `mpl-core` binaries in milliseconds and inspect the raw account state after every step—like seeing that burning an MPL Core NFT doesn't delete the account, it leaves a 1-byte tombstone behind and keeps the protocol creation fee.

## Run it

```bash
cargo test
```
