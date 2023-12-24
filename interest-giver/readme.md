# Interest crank service
This service is responsible for calculating the interest for each account and updating the account balance.
It is incumbent on the service/user/actor to ensure the keypair is valid and has sufficient funds to pay for the transactions.

## Usage
```bash
$ interest-giver <keypair> <program_id>

# Example
$ interest-giver ~/.config/solana/id.json 145CK1g8wC9bYZ5fj6qw5KTrxYAAvCTaosrCdhw15S9u
```