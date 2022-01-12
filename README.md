# Paras NFT Farming Contract

This contract is based from Ref-Finance farming contract. 
We modified the contract to create a farm with staked NFT. The staked NFT will be valued as X amount of FT.

## How to compile
```
$ cd ref-farming
$ ./build_docker.sh
```

## How to use
We have precompiled binary in res/ directory. 

```
PARAS FT testnet: dev-1631277489384-75412609538902
```

### Deploy to testnet (dev)
```sh
near dev-deploy --wasmFile res/ref_farming_release.wasm
```

### Init
```sh
near call --accountId dev-1641987418790-52566958498708 dev-1641987418790-52566958498708 new '{"owner_id":"dev-1641987418790-52566958498708"}'
```

### Create Farm (FT)
```sh
near call --accountId dev-1641987418790-52566958498708 dev-1641987418790-52566958498708 create_simple_farm '{"terms":{"seed_id":"dev-1631277489384-75412609538902","reward_token":"dev-1631277489384-75412609538902","start_at":0,"reward_per_session":"1000000000000000000","session_interval":60},"metadata":{"title":"PARAS Staking","media":"https://paras-cdn.imgix.net/bafybeidoerucqfzyazvyfm5axjixs6vie7ts2myru7g5mu2ub7tlvixpqq?w=800"}}' --depositYocto 19380000000000000000000
```

### Create farm (NFT)
```sh
near call --accountId dev-1641987418790-52566958498708 dev-1641987418790-52566958498708 create_simple_farm '{"terms":{"seed_id":"dev-1631277489384-75412609538902$1","reward_token":"dev-1631277489384-75412609538902","start_at":0,"reward_per_session":"1000000000000000000","session_interval":60},"nft_balance":{"paras-token-v1.testnet@194":"500000000000000000000","paras-token-v1.testnet@177":"100000000000000000000","paras-comic-dev.testnet@6":"200000000000000000000"},"metadata":{"title":"Vitamins Pool","media":"https://cdn.paras.id/tr:w-0.8/bafybeiboxzb5qzwuvkw4vlcubc6sd5vfu532qr6nomzj2dgq7pigh5jfay"}}' --depositYocto 19380000000000000000000
```

### View farm
```sh
near view dev-1641987418790-52566958498708 get_farm '{"farm_id":"dev-1631277489384-75412609538902$1#0"}'
```

### View seed
```sh
near view dev-1641987418790-52566958498708 get_seed_info '{"seed_id":"dev-1631277489384-75412609538902$1"}'
```

### Add reward
```sh
near call --accountId orang.testnet dev-1631277489384-75412609538902 ft_transfer_call '{"receiver_id":"dev-1641987418790-52566958498708","amount":"250000000000000000000000","msg":"dev-1631277489384-75412609538902$1#0"}' --depositYocto 1 --gas 300000000000000
```

### Register as Farmer
```sh
near call --accountId cymac.testnet dev-1641987418790-52566958498708 storage_deposit '{"account_id":"cymac.testnet"}'
--depositYocto 18520000000000000000000
```

### Stake FT
```sh
near call --accountId cymac.testnet dev-1631277489384-75412609538902 ft_transfer_call '{"receiver_id":"dev-1641987418790-52566958498708","amount":"10000000000000000000","msg":""}' --depositYocto 1 --gas 300000000000000
```

### View staked FT
```sh
near view dev-1641987418790-52566958498708 list_user_seeds '{"account_id":"cymac.testnet"}'
```

### Stake NFT
```sh
near call --accountId cymac.testnet paras-token-v1.testnet nft_transfer_call '{"receiver_id":"dev-1641987418790-52566958498708","token_id":"177:5","msg":"dev-1631277489384-75412609538902$1"}' --depositYocto 1 --gas 300000000000000
```

### View staked NFT
```sh
near view dev-1641987418790-52566958498708 list_user_nft_seeds '{"account_id":"cymac.testnet"}'
```

### Unstake FT
```sh
near call --accountId cymac.testnet dev-1641987418790-52566958498708 withdraw_seed '{"seed_id":"dev-1631277489384-75412609538902","amount":"10000000000000000000"}' --depositYocto 1 --gas 100000000000000
```

### Unstake NFT
```sh
near call --accountId cymac.testnet dev-1641987418790-52566958498708 withdraw_nft '{"seed_id":"dev-1631277489384-75412609538902$1","nft_contract_id":"paras-token-v1.testnet","nft_token_id":"177:5"}' --depositYocto 1 --gas 100000000000000
```

### View unclaimed rewards
```sh
near view dev-1641987418790-52566958498708 get_unclaimed_reward '{"account_id":"cymac.testnet","farm_id":"dev-1631277489384-75412609538902$1#0"}'
```

### Claim rewards
```sh
near call --accountId cymac.testnet dev-1641987418790-52566958498708 claim_reward_by_farm '{"farm_id":"dev-1631277489384-75412609538902$1#0"}'
```

### List claimed rewards
```sh
near view dev-1641987418790-52566958498708 list_rewards '{"account_id":"cymac.testnet"}'
```

### Withdraw reward
```sh
near call --accountId cymac.testnet dev-1641987418790-52566958498708 withdraw_reward '{"token_id":"dev-1631277489384-75412609538902"}' --depositYocto 1 --gas 300000000000000
```

### Claim and withdraw reward
```sh
near call --accountId cymac.testnet dev-1641987418790-52566958498708 claim_reward_by_farm_and_withdraw '{"farm_id":"dev-1631277489384-75412609538902$1#0"}' --depositYocto 1 --gas 300000000000000
```

### Claim and withdraw reward by seed
```sh
near call --accountId cymac.testnet dev-1641987418790-52566958498708 claim_reward_by_seed_and_withdraw '{"seed_id":"dev-1631277489384-75412609538902$1","token_id":"dev-1631277489384-75412609538902"}' --depositYocto 1 --gas 300000000000000
```


# Ref Finance Contracts

This mono repo contains the source code for the smart contracts of Ref Finance on [NEAR](https://near.org).

## Contracts

| Contract | Reference | Description |
| - | - | - |
| [test-token](test-token/src/lib.rs) | - | Test token contract |
| [ref-exchange](ref-exchange/src/lib.rs) | [docs](https://ref-finance.gitbook.io/ref-finance/smart-contracts/ref-exchange) | Main exchange contract, that allows to deposit and withdraw tokens, exchange them via various pools |

## Development

1. Install `rustup` via https://rustup.rs/
2. Run the following:

```
rustup default stable
rustup target add wasm32-unknown-unknown
```

### Testing

Contracts have unit tests and also integration tests using NEAR Simulation framework. All together can be run:

```
cd ref-exchange
cargo test --all
```

### Compiling

You can build release version by running next scripts inside each contract folder:

```
cd ref-exchange
./build.sh
```

### Deploying to TestNet

To deploy to TestNet, you can use next command:
```
near dev-deploy
```

This will output on the contract ID it deployed.
