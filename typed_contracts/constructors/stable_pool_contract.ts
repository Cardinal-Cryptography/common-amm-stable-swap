import {CodePromise} from "@polkadot/api-contract";
import type {KeyringPair} from "@polkadot/keyring/types";
import type {ApiPromise} from "@polkadot/api";
import {_genValidGasLimitAndValue, _signAndSend, SignAndSendSuccessResponse} from "@727-ventures/typechain-types";
import type {ConstructorOptions} from "@727-ventures/typechain-types";
import type {WeightV2} from "@polkadot/types/interfaces";
import type * as ArgumentTypes from '../types-arguments/stable_pool_contract';
import { ContractFile } from '../contract-info/stable_pool_contract';
import type BN from 'bn.js';

export default class Constructors {
	readonly nativeAPI: ApiPromise;
	readonly signer: KeyringPair;

	constructor(
		nativeAPI: ApiPromise,
		signer: KeyringPair,
	) {
		this.nativeAPI = nativeAPI;
		this.signer = signer;
	}

	/**
	* newStable
	*
	* @param { Array<ArgumentTypes.AccountId> } tokens,
	* @param { Array<(number | string | BN)> } tokensDecimals,
	* @param { (string | number | BN) } initAmpCoef,
	* @param { ArgumentTypes.AccountId } owner,
	* @param { (number | string | BN) } tradeFee,
	* @param { (number | string | BN) } protocolFee,
	* @param { ArgumentTypes.AccountId | null } feeReceiver,
	*/
   	async "newStable" (
		tokens: Array<ArgumentTypes.AccountId>,
		tokensDecimals: Array<(number | string | BN)>,
		initAmpCoef: (string | number | BN),
		owner: ArgumentTypes.AccountId,
		tradeFee: (number | string | BN),
		protocolFee: (number | string | BN),
		feeReceiver: ArgumentTypes.AccountId | null,
		__options ? : ConstructorOptions,
   	) {
   		const __contract = JSON.parse(ContractFile);
		const code = new CodePromise(this.nativeAPI, __contract, __contract.source.wasm);
		const gasLimit = (await _genValidGasLimitAndValue(this.nativeAPI, __options)).gasLimit as WeightV2;

		const storageDepositLimit = __options?.storageDepositLimit;
			const tx = code.tx["newStable"]!({ gasLimit, storageDepositLimit, value: __options?.value }, tokens, tokensDecimals, initAmpCoef, owner, tradeFee, protocolFee, feeReceiver);
			let response;

			try {
				response = await _signAndSend(this.nativeAPI.registry, tx, this.signer, (event: any) => event);
			}
			catch (error) {
				console.log(error);
			}

		return {
			result: response as SignAndSendSuccessResponse,
			// @ts-ignore
			address: (response as SignAndSendSuccessResponse)!.result!.contract.address.toString(),
		};
	}
	/**
	* newRated
	*
	* @param { Array<ArgumentTypes.AccountId> } tokens,
	* @param { Array<(number | string | BN)> } tokensDecimals,
	* @param { Array<ArgumentTypes.AccountId | null> } externalRates,
	* @param { (number | string | BN) } rateExpirationDurationMs,
	* @param { (string | number | BN) } initAmpCoef,
	* @param { ArgumentTypes.AccountId } owner,
	* @param { (number | string | BN) } tradeFee,
	* @param { (number | string | BN) } protocolFee,
	* @param { ArgumentTypes.AccountId | null } feeReceiver,
	*/
   	async "newRated" (
		tokens: Array<ArgumentTypes.AccountId>,
		tokensDecimals: Array<(number | string | BN)>,
		externalRates: Array<ArgumentTypes.AccountId | null>,
		rateExpirationDurationMs: (number | string | BN),
		initAmpCoef: (string | number | BN),
		owner: ArgumentTypes.AccountId,
		tradeFee: (number | string | BN),
		protocolFee: (number | string | BN),
		feeReceiver: ArgumentTypes.AccountId | null,
		__options ? : ConstructorOptions,
   	) {
   		const __contract = JSON.parse(ContractFile);
		const code = new CodePromise(this.nativeAPI, __contract, __contract.source.wasm);
		const gasLimit = (await _genValidGasLimitAndValue(this.nativeAPI, __options)).gasLimit as WeightV2;

		const storageDepositLimit = __options?.storageDepositLimit;
			const tx = code.tx["newRated"]!({ gasLimit, storageDepositLimit, value: __options?.value }, tokens, tokensDecimals, externalRates, rateExpirationDurationMs, initAmpCoef, owner, tradeFee, protocolFee, feeReceiver);
			let response;

			try {
				response = await _signAndSend(this.nativeAPI.registry, tx, this.signer, (event: any) => event);
			}
			catch (error) {
				console.log(error);
			}

		return {
			result: response as SignAndSendSuccessResponse,
			// @ts-ignore
			address: (response as SignAndSendSuccessResponse)!.result!.contract.address.toString(),
		};
	}
}