const farmingContractName = 'dev-1647599503667-53369781664552'
const nftContractName = 'paras-token-v1.testnet'
const ftContractName = 'dev-1631277489384-75412609538902'
const ownerAccountName = 'orang.testnet'

module.exports = function getConfig(network = 'mainnet') {
	let config = {
		networkId: "testnet",
		nodeUrl: "https://rpc.testnet.near.org",
		walletUrl: "https://wallet.testnet.near.org",
		helperUrl: "https://helper.testnet.near.org",
        farmingContractName: farmingContractName,
        nftContractName: nftContractName,
        ftContractName: ftContractName,
        ownerAccountName: ownerAccountName
	}

	switch (network) {
	case 'testnet':
		config = {
			explorerUrl: "https://explorer.testnet.near.org",
			...config,
			GAS: "200000000000000",
			gas: "200000000000000",
			gas_max: "300000000000000",
			DEFAULT_NEW_ACCOUNT_AMOUNT: "2",
			DEFAULT_NEW_CONTRACT_AMOUNT: "5",
			GUESTS_ACCOUNT_SECRET: "7UVfzoKZL4WZGF98C3Ue7tmmA6QamHCiB1Wd5pkxVPAc7j6jf3HXz5Y9cR93Y68BfGDtMLQ9Q29Njw5ZtzGhPxv",
        }
    }

	return config
}