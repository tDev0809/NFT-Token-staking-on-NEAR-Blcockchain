# Paras NFT Farming Contract

This contract is based from Ref-Finance farming contract. 
We modified the contract to use NFT as FT (seed) amount multiplier.

## How to compile
```
$ cd ref-farming
$ ./build_docker.sh
```

## How to use
We have precompiled binary in res/ directory. 


### Deploy to testnet (dev)
```
near dev-deploy --wasmFile res/ref_farming_release.wasm
```

### Init
```
near call --accountId dev-1636016829431-36818695401642 dev-1636016829431-36818695401642 new '{"owner_id":"dev-1636016829431-36818695401642"}'
```

### Create farm
```
near call --accountId dev-1636016829431-36818695401642 dev-1636016829431-36818695401642 create_simple_farm '{"terms":{"seed_id":"dev-1631277489384-75412609538902$1","reward_token":"dev-1631277489384-75412609538902","start_at":0,"reward_per_session":"1000000000000000000","session_interval":60},"nft_multiplier":{"paras-token-v1.testnet@194":10000,"paras-token-v1.testnet@177":1000,"paras-comic-dev.testnet@6": 100}}' --depositYocto 9380000000000000000000
```

### View farm
```
near view dev-1636016829431-36818695401642 get_farm '{"farm_id":"dev-1631277489384-75412609538902$1#0"}'
```

### View seed
```
near view dev-1636016829431-36818695401642 get_seed_info '{"seed_id":"dev-1631277489384-75412609538902$1"}'
```

### Add reward
```
near call --accountId orang.testnet dev-1631277489384-75412609538902 ft_transfer_call '{"receiver_id":"dev-1636016829431-36818695401642","amount":"250000000000000000000000","msg":"dev-1631277489384-75412609538902$1#0"}' --depositYocto 1 --gas 300000000000000
```

### Register as Farmer
```
near call --accountId cymac.testnet dev-1636016829431-36818695401642 storage_deposit '{"account_id":"cymac.testnet"}'
--depositYocto 18520000000000000000000
```

### Stake FT
```
near call --accountId cymac.testnet dev-1631277489384-75412609538902 ft_transfer_call '{"receiver_id":"dev-1636016829431-36818695401642","amount":"10000000000000000000","msg":"{\"transfer_type\":\"seed\",\"seed_id\":\"dev-1631277489384-75412609538902$1\"}"}' --depositYocto 1 --gas 300000000000000
```

### View staked FT
```
near view dev-1636016829431-36818695401642 list_user_seeds '{"account_id":"cymac.testnet"}'
```

### Stake NFT
```
near call --accountId cymac.testnet paras-token-v1.testnet nft_transfer_call '{"receiver_id":"dev-1636016829431-36818695401642","token_id":"177:5","msg":"dev-1631277489384-75412609538902$1"}' --depositYocto 1 --gas 300000000000000
```

### View staked NFT
```
near view dev-1636016829431-36818695401642 list_user_nft_seeds '{"account_id":"cymac.testnet"}'
```

### Unstake FT
```
near call --accountId cymac.testnet dev-1636016829431-36818695401642 withdraw_seed '{"seed_id":"dev-1631277489384-75412609538902$1","amount":"10000000000000000000"}' --depositYocto 1 --gas 100000000000000
```

### Unstake NFT
```
near call --accountId cymac.testnet dev-1636016829431-36818695401642 withdraw_nft '{"seed_id":"dev-1631277489384-75412609538902$1","nft_contract_id":"paras-token-v1.testnet","nft_token_id":"177:5"}' --depositYocto 1 --gas 100000000000000
```

### View unclaimed rewards
```
near view dev-1636016829431-36818695401642 get_unclaimed_reward '{"account_id":"cymac.testnet","farm_id":"dev-1631277489384-75412609538902$1#0"}'
```

### Claim rewards
```
near call --accountId cymac.testnet dev-1636016829431-36818695401642 claim_reward_by_farm '{"farm_id":"dev-1631277489384-75412609538902$1#0"}'
```

### List claimed rewards
```
near view dev-1636016829431-36818695401642 list_rewards '{"account_id":"cymac.testnet"}'
```

### Withdraw reward
```
near call --accountId cymac.testnet dev-1636016829431-36818695401642 withdraw_reward '{"token_id":"dev-1631277489384-75412609538902"}' --depositYocto 1 --gas 300000000000000
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
