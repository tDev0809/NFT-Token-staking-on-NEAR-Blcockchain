const fs = require("fs");
const BN = require('bn.js');
const fetch = require('node-fetch');
const nearAPI = require('near-api-js');
const { KeyPair, Account, Contract, utils: { format: { parseNearAmount } } } = nearAPI;
const getConfig = require('./config');
const {
	networkId, 
    farmingContractName, nftContractName, ftContractName,
    ownerAccountName,
    contractMethods, gas, gas_max, nodeUrl, walletUrl,
	DEFAULT_NEW_ACCOUNT_AMOUNT, 
	DEFAULT_NEW_CONTRACT_AMOUNT,
	GUESTS_ACCOUNT_SECRET,
} = getConfig('testnet');

const keyStore = new nearAPI.keyStores.UnencryptedFileSystemKeyStore(
    `${process.env.HOME}/.near-credentials/`
)

const near = new nearAPI.Near({
    deps: {
            keyStore: keyStore,
    },
    networkId: networkId,
    keyStore: keyStore,
    nodeUrl: nodeUrl,
    walletUrl: walletUrl
})

const farmingContractAccount = new nearAPI.Account(near.connection, farmingContractName)
const ownerAccount = new nearAPI.Account(near.connection, ownerAccountName)

farmingContractAccount.addAccessKey = (publicKey) =>
	farmingContractAccount.addKey(
		publicKey,
		farmingContractName,
		{
            viewMethods: ["get_seed_info", "list_user_seeds", "get_farm"],
            changeMethods: ["new", "create_simple_farm", "storage_deposit", "force_upgrade_seed", "upgrade_lookup_map", "migrate"]
        },
		parseNearAmount("0.1")
	);

const farmingContract = new nearAPI.Contract(
    farmingContractAccount,
    farmingContractName,
    {
      viewMethods: ["get_seed_info", "list_user_seeds", "list_user_nft_seeds", "get_farm"],
      changeMethods: ["new", "create_simple_farm", "storage_deposit", "force_upgrade_seed", "upgrade_lookup_map", "migrate"]
    }
)

const nftContract = new nearAPI.Contract(
    ownerAccount,
    nftContractName,
    {
      viewMethods: ["nft_token"],
      changeMethods: ["nft_transfer", "nft_transfer_call"]
    }
)

const ftContract = new nearAPI.Contract(
    ownerAccount,
    ftContractName,
    {
      viewMethods: ["ft_balance_of"],
      changeMethods: ["ft_transfer", "ft_transfer_call", "storage_deposit"]
    }
)

async function initContract() {
	try {
		await farmingContract.new({
            owner_id: farmingContractName
        })
	} catch (e) {
        throw e;
	}
	return { farmingContract, farmingContractNAme };
}

const getAccountBalance = async (accountId) => (new nearAPI.Account(connection, accountId)).getAccountBalance();

const initAccount = async(accountId, secret) => {
	account = new nearAPI.Account(connection, accountId);
	const newKeyPair = KeyPair.fromString(secret);
	keyStore.setKey(networkId, accountId, newKeyPair);
	return account;
};

const createOrInitAccount = async(accountId, secret = GUESTS_ACCOUNT_SECRET, amount = DEFAULT_NEW_CONTRACT_AMOUNT) => {
	let account;
	try {
		account = await createAccount(accountId, amount, secret);
	} catch (e) {
		if (!/because it already exists/.test(e.toString())) {
			throw e;
		}
		account = initAccount(accountId, secret);
	}
	return account;
};

async function getAccount(accountId, fundingAmount = DEFAULT_NEW_ACCOUNT_AMOUNT, secret) {
	accountId = accountId || generateUniqueSubAccount();
	const account = new nearAPI.Account(connection, accountId);
	try {
		await account.state();
		return account;
	} catch(e) {
		if (!/does not exist/.test(e.toString())) {
			throw e;
		}
	}
	return await createAccount(accountId, fundingAmount, secret);
};


async function getContract(account) {
	return new Contract(account || contractAccount, contractName, {
		...contractMethods,
		signer: account || undefined
	});
}


const createAccessKeyAccount = (key) => {
	connection.signer.keyStore.setKey(networkId, contractName, key);
	return new Account(connection, contractName);
};

function generateUniqueSubAccount() {
	return `t${Date.now()}.${contractName}`;
}

/// internal
async function createAccount(accountId, fundingAmount = DEFAULT_NEW_ACCOUNT_AMOUNT, secret) {
	const contractAccount = new Account(connection, contractName);
	const newKeyPair = secret ? KeyPair.fromString(secret) : KeyPair.fromRandom('ed25519');
	await contractAccount.createAccount(accountId, newKeyPair.publicKey, new BN(parseNearAmount(fundingAmount)));
	keyStore.setKey(networkId, accountId, newKeyPair);
	return new nearAPI.Account(connection, accountId);
}

module.exports = { 
	near,
	gas,
	gas_max,
	keyStore,
	getContract,
	getAccountBalance,
    farmingContractAccount,
	farmingContractName,
    nftContractName,
    ftContractName,
	ownerAccountName,
    farmingContract,
    nftContract,
    ftContract,
    ownerAccount,
	contractMethods,
	initAccount,
	createOrInitAccount,
	createAccessKeyAccount,
	initContract, 
    getAccount, 
};
