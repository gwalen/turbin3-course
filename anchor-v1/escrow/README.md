# Token-swap escrow

## How it works

The maker deposits token A into a vault and fixes the amount of token B they want in return. Any taker who pays that amount receives the vault's entire token A balance. The basic offer stays open until a taker accepts it or the maker refunds it. Offer terms cannot be updated.

```text
Maker                       Escrow program                         Taker
  |                               |                                  |
  | make(seed, deposit, receive)  |                                  |
  |------------------------------>| create escrow PDA and token vault |
  | token A --------------------->| vault                            |
  |                               |                                  |
  |                               |                             take |
  |                               |<---------------------------------|
  |<------------------------------| token B from taker                |
  |                               | vault's token A ---------------->|
  |<------------------------------| close escrow and vault; return SOL rent
  |                               |                                  |
  | refund (alternative to take)   |                                  |
  |------------------------------>|                                  |
  |<------------------------------| return token A and close accounts|
```

Transfers use `token_interface::transfer_checked`. Mint A and mint B each have a separate token program (`token_program_a` and `token_program_b`), so an offer can swap SPL Token against Token-2022 in either direction, or use the same program for both. Deposits, vault payouts, and vault closure use A's program; payments use B's program. Refunds continue to take one `token_program` for mint A. Both creation instructions check the extension types on both mints and reject:

| Mint extension | Error | Reason |
|---|---|---|
| `TransferFeeConfig`, including a zero current fee | `TransferFeeMint` | Withheld deposit fees prevent vault closure, and payment fees reduce the maker's credited amount. |
| `PermanentDelegate` | `PermanentDelegateMint` | A mint-level delegate can transfer or burn vault tokens without the escrow PDA. |
| `TransferHook`, including a disabled hook | `TransferHookMint` | A hook can require accounts absent from the escrow's token CPIs, and its authority can enable it after creation. |

Both take instructions reject an empty vault with `EmptyVault` before paying token B. This guard prevents settlement for zero tokens; creation-time extension rejection prevents the delegate-driven partial-drain case for new offers. Existing offers created by an earlier version are not revalidated or migrated, so this change does not repair legacy offers with these extensions.

Tests exercise classic SPL Token, fee-free Token-2022 and mixed-program swaps and refunds in both directions, maker-wallet and classic payment-ATA authority reassignment, memo-required payments, transfer-fee rejection, permanent-delegate and transfer-hook rejection on either mint, and empty-vault settlement rejection. Token amounts use six decimal places. Other Token-2022 extensions are outside the test coverage.

### Accounts

Each maker can create several offers by choosing different `seed` values. Each escrow account records its maker, the two mints, the requested token B amount, and its PDA bump. The vault's token balance is the source of truth for the token A amount.

| Account | Address / contents | Authority |
|---|---|---|
| Basic escrow | PDA seeds `[b"escrow", maker, seed.to_le_bytes()]` | Escrow program |
| Timed escrow | PDA seeds `[b"timed_escrow", maker, seed.to_le_bytes()]`; also stores `expires_at` | Escrow program |
| Vault | Associated token account (ATA) for mint A, owned by its escrow PDA | Escrow PDA |
| Maker token accounts | Maker's ATAs for A and B | Maker, or the authority the maker assigned to the classic B ATA |
| Taker token accounts | Taker's ATAs for A and B | Taker |

The maker funds the escrow and vault accounts at creation. A successful take or refund closes both accounts and returns their SOL rent to the maker. The taker pays to create missing destination ATAs during a take. A refund can recreate the maker's A account if the maker closed it after depositing.

The escrow constraints bind the maker and both mints to the stored offer. Taking accepts the stored maker as a writable rent recipient regardless of which program owns that wallet; `has_one = maker` still enforces its identity. Changing the maker wallet's program owner cannot block a timed take.

Both take instructions pay the canonical B ATA derived from the stored maker, mint B, and B's token program. If that account is empty, the taker funds its creation. Otherwise, its token program and mint are validated, and payment goes to that same address even if the maker reassigned its classic SPL Token authority. That ownership choice belongs to the maker; the program preserves it instead of letting it block settlement.

The make and take account lists now carry both token programs; clients must rebuild their instructions from the updated IDL. Both take instructions require the canonical SPL Memo program account and invoke it immediately before Token-2022 payments, so enabling required incoming memos on the maker's B account cannot block payment. The vault can pay out only when this program signs with the escrow PDA's seeds. Token payment, vault payout, and account closure occur in one transaction, so a failed payment rolls back the swap.

### Timed offers

`make_timed(seed, deposit, receive, expires_at)` creates a separate timed offer. `expires_at` is a Unix timestamp in seconds and must be in the future according to the on-chain `Clock`.

| Time | `take_expired` | `refund_expired` |
|---|---|---|
| `now < expires_at` | Taker can accept the offer | Maker gets `NotExpired` |
| `now >= expires_at` | Taker gets `Expired` | Maker can refund |

Despite its name, `take_expired` is the take instruction for a timed offer **before** expiry. At exactly `expires_at`, taking fails and refunding succeeds. The maker's deposit therefore stays locked while the offer can be taken.

A refund does not run automatically. The maker sends `refund_expired` after expiry; the funds remain in the vault until that transaction succeeds. Basic instructions accept `Escrow` accounts, and timed instructions accept `TimedEscrow` accounts with a different seed prefix.

Successful creation emits `EscrowMade` or `TimedEscrowMade`. Basic and timed takes emit `EscrowTaken`; refunds emit `EscrowRefunded`.

## Build and test

Install [just](https://just.systems/), Anchor CLI 1.2.1, and the Solana CLI/SBF build tools. Rust 1.97.1 is selected by `rust-toolchain.toml`.

```console
brew install just
```

Run commands from this project directory:

```console
cd anchor-v1/escrow
just build
just rust
```

`just build` runs `NO_DNA=1 anchor build`, preserving Anchor's default sBPF v3 architecture. The generated program is loaded from `target/deploy/escrow.so` by the Rust tests, so build before running tests after an on-chain code change.

The tests run in-process on LiteSVM 0.18.2, using deterministic funded maker and taker fixtures. This replaces the original plan's LiteSVM 0.10.0 so the runtime can load the sBPF v3 artifact. The test harness uses compatible Solana message 4.x and transaction 4.x crates; `Cargo.lock` records the resolved dependency set. The SPL Memo CPI uses the `memo` feature of `anchor-spl`, with `spl-memo-interface` resolved in the lockfile. Initial dependency downloads require crates.io access.

To display transaction logs:

```console
just rust-logs
```

The numbered test files cover basic creation, swapping, refunds, account substitution, rollback, independent offers, timed expiry boundaries, mint-extension rejection, empty-vault settlement rejection, maker-wallet reassignment, memo-required Token-2022 payments, and mixed-program swaps and refunds. The liveness tests use real System Program assignment, classic SPL Token account-authority reassignment, and Token-2022 reallocation and memo-enabling instructions. Timed tests set LiteSVM's `Clock` sysvar directly. Empty-vault tests rewrite a token-account snapshot to represent an already emptied offer. The tests do not sleep or need a running validator.
