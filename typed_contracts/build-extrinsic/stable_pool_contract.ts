/* This file is auto-generated */

import type { ContractPromise } from '@polkadot/api-contract';
import type { GasLimit, GasLimitAndRequiredValue } from '@727-ventures/typechain-types';
import { buildSubmittableExtrinsic } from '@727-ventures/typechain-types';
import type * as ArgumentTypes from '../types-arguments/stable_pool_contract';
import type BN from 'bn.js';
import type { ApiPromise } from '@polkadot/api';



export default class Methods {
	readonly __nativeContract : ContractPromise;
	readonly __apiPromise: ApiPromise;

	constructor(
		nativeContract : ContractPromise,
		apiPromise: ApiPromise,
	) {
		this.__nativeContract = nativeContract;
		this.__apiPromise = apiPromise;
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::addLiquidity", [minShareAmount, amounts, to], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::removeLiquidityByShares", [shares, minAmounts, to], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::removeLiquidityByAmounts", [maxShareAmount, amounts, to], __options);
	}

	/**
	 * forceUpdateRates
	 *
	*/
	"forceUpdateRates" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::forceUpdateRates", [], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::swapExactIn", [tokenIn, tokenOut, tokenInAmount, minTokenOutAmount, to], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::swapExactOut", [tokenIn, tokenOut, tokenOutAmount, maxTokenInAmount, to], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::swapReceived", [tokenIn, tokenOut, minTokenOutAmount, to], __options);
	}

	/**
	 * setFeeReceiver
	 *
	 * @param { ArgumentTypes.AccountId | null } feeReceiver,
	*/
	"setFeeReceiver" (
		feeReceiver: ArgumentTypes.AccountId | null,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::setFeeReceiver", [feeReceiver], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::setFees", [tradeFee, protocolFee], __options);
	}

	/**
	 * setAmpCoef
	 *
	 * @param { (string | number | BN) } ampCoef,
	*/
	"setAmpCoef" (
		ampCoef: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::setAmpCoef", [ampCoef], __options);
	}

	/**
	 * tokens
	 *
	*/
	"tokens" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::tokens", [], __options);
	}

	/**
	 * reserves
	 *
	*/
	"reserves" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::reserves", [], __options);
	}

	/**
	 * ampCoef
	 *
	*/
	"ampCoef" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::ampCoef", [], __options);
	}

	/**
	 * fees
	 *
	*/
	"fees" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::fees", [], __options);
	}

	/**
	 * feeReceiver
	 *
	*/
	"feeReceiver" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::feeReceiver", [], __options);
	}

	/**
	 * tokenRates
	 *
	*/
	"tokenRates" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::tokenRates", [], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::getSwapAmountOut", [tokenIn, tokenOut, tokenInAmount], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::getSwapAmountIn", [tokenIn, tokenOut, tokenOutAmount], __options);
	}

	/**
	 * getMintLiquidityForAmounts
	 *
	 * @param { Array<(string | number | BN)> } amounts,
	*/
	"getMintLiquidityForAmounts" (
		amounts: Array<(string | number | BN)>,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::getMintLiquidityForAmounts", [amounts], __options);
	}

	/**
	 * getAmountsForLiquidityMint
	 *
	 * @param { (string | number | BN) } liquidity,
	*/
	"getAmountsForLiquidityMint" (
		liquidity: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::getAmountsForLiquidityMint", [liquidity], __options);
	}

	/**
	 * getBurnLiquidityForAmounts
	 *
	 * @param { Array<(string | number | BN)> } amounts,
	*/
	"getBurnLiquidityForAmounts" (
		amounts: Array<(string | number | BN)>,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::getBurnLiquidityForAmounts", [amounts], __options);
	}

	/**
	 * getAmountsForLiquidityBurn
	 *
	 * @param { (string | number | BN) } liquidity,
	*/
	"getAmountsForLiquidityBurn" (
		liquidity: (string | number | BN),
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "stablePool::getAmountsForLiquidityBurn", [liquidity], __options);
	}

	/**
	 * totalSupply
	 *
	*/
	"totalSupply" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::totalSupply", [], __options);
	}

	/**
	 * balanceOf
	 *
	 * @param { ArgumentTypes.AccountId } owner,
	*/
	"balanceOf" (
		owner: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::balanceOf", [owner], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::allowance", [owner, spender], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::transfer", [to, value, data], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::transferFrom", [from, to, value, data], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::approve", [spender, value], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::increaseAllowance", [spender, deltaValue], __options);
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
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22::decreaseAllowance", [spender, deltaValue], __options);
	}

	/**
	 * tokenName
	 *
	*/
	"tokenName" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22Metadata::tokenName", [], __options);
	}

	/**
	 * tokenSymbol
	 *
	*/
	"tokenSymbol" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22Metadata::tokenSymbol", [], __options);
	}

	/**
	 * tokenDecimals
	 *
	*/
	"tokenDecimals" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "psp22Metadata::tokenDecimals", [], __options);
	}

	/**
	 * getOwner
	 *
	*/
	"getOwner" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable2Step::getOwner", [], __options);
	}

	/**
	 * getPendingOwner
	 *
	*/
	"getPendingOwner" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable2Step::getPendingOwner", [], __options);
	}

	/**
	 * transferOwnership
	 *
	 * @param { ArgumentTypes.AccountId } newOwner,
	*/
	"transferOwnership" (
		newOwner: ArgumentTypes.AccountId,
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable2Step::transferOwnership", [newOwner], __options);
	}

	/**
	 * acceptOwnership
	 *
	*/
	"acceptOwnership" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable2Step::acceptOwnership", [], __options);
	}

	/**
	 * renounceOwnership
	 *
	*/
	"renounceOwnership" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable2Step::renounceOwnership", [], __options);
	}

	/**
	 * ensureOwner
	 *
	*/
	"ensureOwner" (
		__options: GasLimit,
	){
		return buildSubmittableExtrinsic( this.__apiPromise, this.__nativeContract, "ownable2Step::ensureOwner", [], __options);
	}

}