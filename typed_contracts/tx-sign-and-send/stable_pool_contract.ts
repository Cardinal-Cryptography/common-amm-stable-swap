/* This file is auto-generated */

import type { ContractPromise } from '@polkadot/api-contract';
import type { KeyringPair } from '@polkadot/keyring/types';
import type { ApiPromise } from '@polkadot/api';
import type { GasLimit, GasLimitAndRequiredValue, Result } from '@727-ventures/typechain-types';
import { txSignAndSend } from '@727-ventures/typechain-types';
import type * as ArgumentTypes from '../types-arguments/stable_pool_contract';
import type BN from 'bn.js';
// @ts-ignore
import type {EventRecord} from "@polkadot/api/submittable";
import {decodeEvents} from "../shared/utils";
import EVENT_DATA_TYPE_DESCRIPTIONS from '../event-data/stable_pool_contract.json';


export default class Methods {
	readonly __nativeContract : ContractPromise;
	readonly __keyringPair : KeyringPair;
	readonly __apiPromise: ApiPromise;

	constructor(
		apiPromise: ApiPromise,
		nativeContract : ContractPromise,
		keyringPair : KeyringPair,
	) {
		this.__apiPromise = apiPromise;
		this.__nativeContract = nativeContract;
		this.__keyringPair = keyringPair;
	}

	/**
	* addLiquidity
	*
	* @param { (string | number | BN) } minShareAmount,
	* @param { Array<(string | number | BN)> } amounts,
	* @param { ArgumentTypes.AccountId } to,
	*/
	"addLiquidity" (
		minShareAmount: (string | number | BN),
		amounts: Array<(string | number | BN)>,
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::addLiquidity", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [minShareAmount, amounts, to], __options);
	}

	/**
	* removeLiquidityByShares
	*
	* @param { (string | number | BN) } shares,
	* @param { Array<(string | number | BN)> } minAmounts,
	* @param { ArgumentTypes.AccountId } to,
	*/
	"removeLiquidityByShares" (
		shares: (string | number | BN),
		minAmounts: Array<(string | number | BN)>,
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::removeLiquidityByShares", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [shares, minAmounts, to], __options);
	}

	/**
	* removeLiquidityByAmounts
	*
	* @param { (string | number | BN) } maxShareAmount,
	* @param { Array<(string | number | BN)> } amounts,
	* @param { ArgumentTypes.AccountId } to,
	*/
	"removeLiquidityByAmounts" (
		maxShareAmount: (string | number | BN),
		amounts: Array<(string | number | BN)>,
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::removeLiquidityByAmounts", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [maxShareAmount, amounts, to], __options);
	}

	/**
	* forceUpdateRates
	*
	*/
	"forceUpdateRates" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::forceUpdateRates", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* swapExactIn
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } tokenInAmount,
	* @param { (string | number | BN) } minTokenOutAmount,
	* @param { ArgumentTypes.AccountId } to,
	*/
	"swapExactIn" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		tokenInAmount: (string | number | BN),
		minTokenOutAmount: (string | number | BN),
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::swapExactIn", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [tokenIn, tokenOut, tokenInAmount, minTokenOutAmount, to], __options);
	}

	/**
	* swapExactOut
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } tokenOutAmount,
	* @param { (string | number | BN) } maxTokenInAmount,
	* @param { ArgumentTypes.AccountId } to,
	*/
	"swapExactOut" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		tokenOutAmount: (string | number | BN),
		maxTokenInAmount: (string | number | BN),
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::swapExactOut", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [tokenIn, tokenOut, tokenOutAmount, maxTokenInAmount, to], __options);
	}

	/**
	* swapReceived
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } minTokenOutAmount,
	* @param { ArgumentTypes.AccountId } to,
	*/
	"swapReceived" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		minTokenOutAmount: (string | number | BN),
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::swapReceived", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [tokenIn, tokenOut, minTokenOutAmount, to], __options);
	}

	/**
	* setFeeReceiver
	*
	* @param { ArgumentTypes.AccountId | null } feeReceiver,
	*/
	"setFeeReceiver" (
		feeReceiver: ArgumentTypes.AccountId | null,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::setFeeReceiver", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [feeReceiver], __options);
	}

	/**
	* setFees
	*
	* @param { (number | string | BN) } tradeFee,
	* @param { (number | string | BN) } protocolFee,
	*/
	"setFees" (
		tradeFee: (number | string | BN),
		protocolFee: (number | string | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::setFees", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [tradeFee, protocolFee], __options);
	}

	/**
	* setAmpCoef
	*
	* @param { (string | number | BN) } ampCoef,
	*/
	"setAmpCoef" (
		ampCoef: (string | number | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::setAmpCoef", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [ampCoef], __options);
	}

	/**
	* tokens
	*
	*/
	"tokens" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::tokens", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* reserves
	*
	*/
	"reserves" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::reserves", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* ampCoef
	*
	*/
	"ampCoef" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::ampCoef", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* fees
	*
	*/
	"fees" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::fees", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* feeReceiver
	*
	*/
	"feeReceiver" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::feeReceiver", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* tokenRates
	*
	*/
	"tokenRates" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::tokenRates", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* getSwapAmountOut
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } tokenInAmount,
	*/
	"getSwapAmountOut" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		tokenInAmount: (string | number | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::getSwapAmountOut", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [tokenIn, tokenOut, tokenInAmount], __options);
	}

	/**
	* getSwapAmountIn
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } tokenOutAmount,
	*/
	"getSwapAmountIn" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		tokenOutAmount: (string | number | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::getSwapAmountIn", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [tokenIn, tokenOut, tokenOutAmount], __options);
	}

	/**
	* getMintLiquidityForAmounts
	*
	* @param { Array<(string | number | BN)> } amounts,
	*/
	"getMintLiquidityForAmounts" (
		amounts: Array<(string | number | BN)>,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::getMintLiquidityForAmounts", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [amounts], __options);
	}

	/**
	* getAmountsForLiquidityMint
	*
	* @param { (string | number | BN) } liquidity,
	*/
	"getAmountsForLiquidityMint" (
		liquidity: (string | number | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::getAmountsForLiquidityMint", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [liquidity], __options);
	}

	/**
	* getBurnLiquidityForAmounts
	*
	* @param { Array<(string | number | BN)> } amounts,
	*/
	"getBurnLiquidityForAmounts" (
		amounts: Array<(string | number | BN)>,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::getBurnLiquidityForAmounts", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [amounts], __options);
	}

	/**
	* getAmountsForLiquidityBurn
	*
	* @param { (string | number | BN) } liquidity,
	*/
	"getAmountsForLiquidityBurn" (
		liquidity: (string | number | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "stablePool::getAmountsForLiquidityBurn", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [liquidity], __options);
	}

	/**
	* totalSupply
	*
	*/
	"totalSupply" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22::totalSupply", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* balanceOf
	*
	* @param { ArgumentTypes.AccountId } owner,
	*/
	"balanceOf" (
		owner: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22::balanceOf", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [owner], __options);
	}

	/**
	* allowance
	*
	* @param { ArgumentTypes.AccountId } owner,
	* @param { ArgumentTypes.AccountId } spender,
	*/
	"allowance" (
		owner: ArgumentTypes.AccountId,
		spender: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22::allowance", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [owner, spender], __options);
	}

	/**
	* transfer
	*
	* @param { ArgumentTypes.AccountId } to,
	* @param { (string | number | BN) } value,
	* @param { Array<(number | string | BN)> } data,
	*/
	"transfer" (
		to: ArgumentTypes.AccountId,
		value: (string | number | BN),
		data: Array<(number | string | BN)>,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22::transfer", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [to, value, data], __options);
	}

	/**
	* transferFrom
	*
	* @param { ArgumentTypes.AccountId } from,
	* @param { ArgumentTypes.AccountId } to,
	* @param { (string | number | BN) } value,
	* @param { Array<(number | string | BN)> } data,
	*/
	"transferFrom" (
		from: ArgumentTypes.AccountId,
		to: ArgumentTypes.AccountId,
		value: (string | number | BN),
		data: Array<(number | string | BN)>,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22::transferFrom", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [from, to, value, data], __options);
	}

	/**
	* approve
	*
	* @param { ArgumentTypes.AccountId } spender,
	* @param { (string | number | BN) } value,
	*/
	"approve" (
		spender: ArgumentTypes.AccountId,
		value: (string | number | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22::approve", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [spender, value], __options);
	}

	/**
	* increaseAllowance
	*
	* @param { ArgumentTypes.AccountId } spender,
	* @param { (string | number | BN) } deltaValue,
	*/
	"increaseAllowance" (
		spender: ArgumentTypes.AccountId,
		deltaValue: (string | number | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22::increaseAllowance", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [spender, deltaValue], __options);
	}

	/**
	* decreaseAllowance
	*
	* @param { ArgumentTypes.AccountId } spender,
	* @param { (string | number | BN) } deltaValue,
	*/
	"decreaseAllowance" (
		spender: ArgumentTypes.AccountId,
		deltaValue: (string | number | BN),
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22::decreaseAllowance", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [spender, deltaValue], __options);
	}

	/**
	* tokenName
	*
	*/
	"tokenName" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22Metadata::tokenName", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* tokenSymbol
	*
	*/
	"tokenSymbol" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22Metadata::tokenSymbol", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* tokenDecimals
	*
	*/
	"tokenDecimals" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "psp22Metadata::tokenDecimals", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* getOwner
	*
	*/
	"getOwner" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "ownable2Step::getOwner", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* getPendingOwner
	*
	*/
	"getPendingOwner" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "ownable2Step::getPendingOwner", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* transferOwnership
	*
	* @param { ArgumentTypes.AccountId } newOwner,
	*/
	"transferOwnership" (
		newOwner: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "ownable2Step::transferOwnership", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [newOwner], __options);
	}

	/**
	* acceptOwnership
	*
	*/
	"acceptOwnership" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "ownable2Step::acceptOwnership", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* renounceOwnership
	*
	*/
	"renounceOwnership" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "ownable2Step::renounceOwnership", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

	/**
	* ensureOwner
	*
	*/
	"ensureOwner" (
		__options ? : GasLimit,
	){
		return txSignAndSend( this.__apiPromise, this.__nativeContract, this.__keyringPair, "ownable2Step::ensureOwner", (events: EventRecord) => {
			return decodeEvents(events, this.__nativeContract, EVENT_DATA_TYPE_DESCRIPTIONS);
		}, [], __options);
	}

}