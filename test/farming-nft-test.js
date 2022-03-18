const fs = require('fs')
const assert = require('assert')
const testUtils = require('./test-utils')
const nearAPI = require('near-api-js')
const JSBI = require('jsbi')
const { error } = require('console')

const {
	gas,
    gas_max,
    farmingContractAccount,
    farmingContract,
    farmingContractName,
    nftContract,
    nftContractName,
    ftContract,
    ftContractName,
    ownerAccount,
    ownerAccountName,
	getAccount,
	createOrInitAccount,
	getAccountBalance,
} = testUtils

describe('NFT Farming Contract', function () {
	this.timeout(60000)

    it('should be deployed', async function () {
		const state = await farmingContractAccount.state();
		try {
			await farmingContract.new({
                owner_id: farmingContractName
			})
		} catch (e) {
			if (!/contract has already been initialized/.test(e.toString())) {
				console.warn(e)
			}
		}

		assert.notStrictEqual(state.code_hash, '11111111111111111111111111111111')
	});

    it('should be upgraded', async function () {
        try {
			await farmingContract.migrate({})
		} catch (e) {
            console.warn(e)
		}
    })

    it('should force upgrade', async function () {
        try {
			await farmingContract.force_upgrade_seed({
                args: {
                    seed_id: ftContractName
                },
                gas: gas_max
            })

			await farmingContract.force_upgrade_seed({
                args: {
                    seed_id: `${nftContractName}`
                },
                gas: gas_max
            })

			await farmingContract.force_upgrade_seed({
                args: {
                    seed_id: `${nftContractName}-7777`
                },
                gas: gas_max
            })
		} catch (e) {
            console.warn(e)
		}
    })
    

    it('should create simple farm ft staking', async function () {
        try {
            const seed = await farmingContract.get_seed_info({
                seed_id: ftContractName
            })
            const farm = await farmingContract.get_farm({
                farm_id: `${ftContractName}#0`
            })
            console.log(seed)
            console.log(farm)
            if (seed) return
			await farmingContract.create_simple_farm({
                terms: {
                    seed_id: ftContractName,
                    reward_token: ftContractName,
                    start_at: 0,
                    reward_per_session: "1000000000000000000",
                    session_interval: 60
                },
                metadata: {
                    title: "PARAS Staking",
                    media: "https://paras-cdn.imgix.net/bafybeidoerucqfzyazvyfm5axjixs6vie7ts2myru7g5mu2ub7tlvixpqq?w=800"
                }
			},
            gas,
            deposit = "19380000000000000000000"
            )
		} catch (e) {
            console.warn(e)
		}
    })

    it('should create simple farm with nft staking', async function () {
        try {
            const seed = await farmingContract.get_seed_info({
                seed_id: nftContractName
            })
            const farm = await farmingContract.get_farm({
                farm_id: `${nftContractName}#0`
            })
            console.log(seed)
            console.log(farm)
            if (seed) return
			await farmingContract.create_simple_farm({
                terms: {
                    seed_id: nftContractName,
                    reward_token: ftContractName,
                    start_at: 0,
                    reward_per_session: "1000000000000000000",
                    session_interval: 60
                },
                nft_balance: {
                    "paras-token-v1.testnet@449":"200000000000000000000"
                },
                metadata: {
                    title: "Key to Paras Testrun",
                    media: "https://paras-cdn.imgix.net/bafybeidoerucqfzyazvyfm5axjixs6vie7ts2myru7g5mu2ub7tlvixpqq?w=800"
                }
			},
            gas,
            deposit = "19380000000000000000000"
            );
		} catch (e) {
            console.warn(e)
		}
    });

    it('should create simple farm with nft staking (7777 NFTs)', async function () {
        try {
            const seed = await farmingContract.get_seed_info({
                seed_id: `${nftContractName}-7777`
            })
            const farm = await farmingContract.get_farm({
                farm_id: `${nftContractName}-7777#0`
            })
            console.log(seed)
            console.log(farm)
            if (seed) return

            let nft_balance = {}
            for (let i = 0; i < 7777; i++) {
                nft_balance[`${nftContractName}@${i}`] = (10 ** 18).toString()
            }
			await farmingContract.create_simple_farm({
                terms: {
                    seed_id: `${nftContractName}-7777`,
                    reward_token: ftContractName,
                    start_at: 0,
                    reward_per_session: "1000000000000000000",
                    session_interval: 60
                },
                nft_balance: nft_balance,
                metadata: {
                    title: "7777 NFTs",
                    media: "https://paras-cdn.imgix.net/bafybeidoerucqfzyazvyfm5axjixs6vie7ts2myru7g5mu2ub7tlvixpqq?w=800"
                }
			},
            "300000000000000",
            deposit = "4000000000000000000000000"
            );
		} catch (e) {
            console.warn(e)
		}
    });

    it('should ft storage deposit', async function () {
        try {
            await farmingContractAccount.functionCall({
                contractId: ftContractName,
                methodName: 'storage_deposit',
                args: {
                    accountId: farmingContractName
                },
                gas: gas,
                attachedDeposit: "1250000000000000000000"
            })
		} catch (e) {
            console.warn(e)
		}
    });

    it('should farming storage deposit', async function () {
        try {
            await ownerAccount.functionCall({
                contractId: farmingContractName,
                methodName: 'storage_deposit',
                args: {
                    accountId: ownerAccountName
                },
                gas: gas,
                attachedDeposit: "100000000000000000000000"
            })
		} catch (e) {
            console.warn(e)
		}
    });

    it('should ft stake', async function () {
        try {
            const user_seeds = await farmingContract.list_user_seeds({
                account_id: ownerAccountName
            })

            if (
                user_seeds[ftContractName] && 
                parseInt(user_seeds[ftContractName]) > 10000000000000000000n
            ) return

            await ftContract.ft_transfer_call(
                {
                    args: {
                        receiver_id: farmingContractName,
                        amount: "100000000000000000000",
                        msg: ""
                    },
                    gas: gas,
                    amount: "1"
                },
            )
		} catch (e) {
            console.warn(e)
		}
    });
    
    it('should ft unstake', async function () {
        try {
            const ownerAccountBalance = await ftContract.ft_balance_of({
                account_id: ownerAccountName
            })
            console.log('ownerAccountBalance', ownerAccountBalance)

            await ownerAccount.functionCall({
                contractId: farmingContractName,
                methodName: 'withdraw_seed',
                args: {
                    seed_id: ftContractName,
                    amount: "10000000000000000000",
                },
                gas: gas,
                attachedDeposit: "1"
            })

            const ownerAccountBalanceAfter = await ftContract.ft_balance_of({
                account_id: ownerAccountName
            })

            if (
                JSBI.lessThan(
                    JSBI.subtract(
                        JSBI.BigInt(ownerAccountBalanceAfter),
                        JSBI.BigInt(ownerAccountBalance)
                        ),
                     JSBI.BigInt("10000000000000000000")
                )
            ) throw Error('amount not the same')

		} catch (e) {
            console.warn(e)
		}
    });

    it('should nft stake with gas_max', async function () {
        try {
            const token_id = "251:46"
            const seed_id = `${nftContractName}-7777`

            const user_nft_seeds = await farmingContract.list_user_nft_seeds({
                account_id: ownerAccountName
            })

            if (user_nft_seeds[seed_id] && user_nft_seeds[seed_id].includes(token_id)) {
                console.log('Already staked nft')
                return
            }

            await nftContract.nft_transfer_call(
                {
                    args: {
                        receiver_id: farmingContractName,
                        token_id: token_id,
                        msg: seed_id
                    },
                    gas: gas_max,
                    amount: "1"
                },
            )
		} catch (e) {
            console.warn(e)
		}
    });

    it('should nft unstake with gas_max', async function () {
        try {
            const token_id = "251:46"
            const contractTokenId = `${nftContractName}@${token_id}`
            const seed_id = `${nftContractName}-7777`

            const user_nft_seeds = await farmingContract.list_user_nft_seeds({
                account_id: ownerAccountName
            })
            console.log(user_nft_seeds)

            if (user_nft_seeds[seed_id] && !user_nft_seeds[seed_id].includes(contractTokenId)) {
                throw Error('No NFT staked')
            }

            await ownerAccount.functionCall({
                contractId: farmingContractName,
                methodName: 'withdraw_nft',
                args: {
                    seed_id: seed_id,
                    nft_contract_id: nftContractName,
                    nft_token_id: token_id
                },
                gas: gas_max,
                attachedDeposit: "1"
            })
		} catch (e) {
            console.warn(e)
		}
    });

    it('should nft stake', async function () {
        try {
            const token_id = "251:46"
            const seed_id = `${nftContractName}-7777`

            const user_nft_seeds = await farmingContract.list_user_nft_seeds({
                account_id: ownerAccountName
            })

            if (user_nft_seeds[seed_id] && user_nft_seeds[seed_id].includes(token_id)) {
                console.log('Already staked nft')
                return
            }

            await nftContract.nft_transfer_call(
                {
                    args: {
                        receiver_id: farmingContractName,
                        token_id: token_id,
                        msg: seed_id
                    },
                    gas: gas,
                    amount: "1"
                },
            )
		} catch (e) {
            console.warn(e)
		}
    });

    it('should nft unstake', async function () {
        try {
            const token_id = "251:46"
            const contractTokenId = `${nftContractName}@${token_id}`
            const seed_id = `${nftContractName}-7777`

            const user_nft_seeds = await farmingContract.list_user_nft_seeds({
                account_id: ownerAccountName
            })

            if (user_nft_seeds[seed_id] && !user_nft_seeds[seed_id].includes(contractTokenId)) {
                throw Error('No NFT staked')
            }

            await ownerAccount.functionCall({
                contractId: farmingContractName,
                methodName: 'withdraw_nft',
                args: {
                    seed_id: seed_id,
                    nft_contract_id: nftContractName,
                    nft_token_id: token_id
                },
                gas: gas,
                attachedDeposit: "1"
            })
		} catch (e) {
            console.warn(e)
		}
    });

    // it('should upgrade farm', async function () {
    //     try {
    //         const seed_id = `${nftContractName}-7777`

    //         // upgrade
    //         await farmingContract.force_upgrade_seed({
    //             args: {
    //                 seed_id: seed_id
    //             },
    //             gas: gas_max
    //         })

    //         const seed_info = await farmingContract.get_seed_info({
    //             seed_id: seed_id
    //         })
    //         console.log(seed_info)
    //     } catch (e) {
    //         console.warn(e)
    //     }
    // });

    // it('should add more nft_balance', async function () {
    //     // run after upgrade
    //     try {
    //         const seed_id = `${nftContractName}-7777`
    //         let seed_info = await farmingContract.get_seed_info({seed_id: seed_id})
    //         let nft_balance = seed_info.nft_balance
    //         let nft_balance_keys = Object.keys(nft_balance)
    //         let limit = 100

    //         for (let i = 0; i < nft_balance_keys.length; i += limit) {
    //             console.log(i)
    //             let nft_balance_sample = {}
    //             for (let j = i; j < i + limit; j++) {
    //                 nft_balance_sample[nft_balance_keys[j]] = nft_balance[nft_balance_keys[j]]
    //             }

    //             await farmingContract.upgrade_lookup_map({
    //                 args: {
    //                     seed_id: seed_id,
    //                     nft_balance: nft_balance_sample
    //                 },
    //                 gas: gas_max
    //             })
    //         }
    //     } catch (e) {
    //         console.warn(e)
    //     }
    // });
})