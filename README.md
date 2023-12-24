# What is this?
This is a smart contract to create a vault for a user to deposit USDC and earn interest on it. The interest is calculated and deposited once per month.
The cron element of this contract is expected to be provided by cron or a similar service.

## Content
- Smart contract for the vault, deposits, withdrawals, and interest deposits (./programs/gfx-token-vaults/src/lib.rs)
- Rust service to invoke interest instruction (./interest-giver/src/main.rs)

## Brief
Write an anchor smart contract that allows wallets to create a USDC (or any SPL token) vault, transfer tokens to it, and withdraw from it.

Write an accompanying rust service that crawls the user vaults once per month and transfers 1% of the currently staked amount to the vault in interest.

If possible:
- use latest anchor packages
- use async rust functions in the service (solana_client::non_blocking)

The interest deposit should be a permission-less instruction where the check for how much amount should be transferred should be in the program logic so that any client can call the crank.

## Helpful commands
```bash
# Build the program
$ anchor build

# Deploy the program
$ anchor deploy

# Test the program
$ anchor test # tests are in ./programs/gfx-token-vaults/tests

# Run the rust service
$ cd ./interest-giver && cargo run interest-giver -- ~/.config/solana/id.json 145CK1g8wC9bYZ5fj6qw5KTrxYAAvCTaosrCdhw15S9u
```