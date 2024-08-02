/* This file is auto-generated */

import type { ContractPromise } from '@polkadot/api-contract';
import type { ApiPromise } from '@polkadot/api';
import type { GasLimit, GasLimitAndRequiredValue, Result } from '@727-ventures/typechain-types';
import type { QueryReturnType } from '@727-ventures/typechain-types';
import { queryJSON, queryOkJSON, handleReturnType } from '@727-ventures/typechain-types';
import type * as ArgumentTypes from '../types-arguments/stable_pool_contract';
import type * as ReturnTypes from '../types-returns/stable_pool_contract';
import type BN from 'bn.js';
//@ts-ignore
import {ReturnNumber} from '@727-ventures/typechain-types';
import {getTypeDescription} from './../shared/utils';
import DATA_TYPE_DESCRIPTIONS from '../data/stable_pool_contract.json';


export default class Methods {
	readonly __nativeContract : ContractPromise;
	readonly __apiPromise: ApiPromise;
	readonly __callerAddress : string;

	constructor(
		nativeContract : ContractPromise,
		nativeApi : ApiPromise,
		callerAddress : string,
	) {
		this.__nativeContract = nativeContract;
		this.__callerAddress = callerAddress;
		this.__apiPromise = nativeApi;
	}

	/**
	* addLiquidity
	*
	* @param { (string | number | BN) } minShareAmount,
	* @param { Array<(string | number | BN)> } amounts,
	* @param { ArgumentTypes.AccountId } to,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"addLiquidity" (
		minShareAmount: (string | number | BN),
		amounts: Array<(string | number | BN)>,
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::addLiquidity", [minShareAmount, amounts, to], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* removeLiquidityByShares
	*
	* @param { (string | number | BN) } shares,
	* @param { Array<(string | number | BN)> } minAmounts,
	* @param { ArgumentTypes.AccountId } to,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"removeLiquidityByShares" (
		shares: (string | number | BN),
		minAmounts: Array<(string | number | BN)>,
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::removeLiquidityByShares", [shares, minAmounts, to], __options , (result) => { return handleReturnType(result, getTypeDescription(26, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* removeLiquidityByAmounts
	*
	* @param { (string | number | BN) } maxShareAmount,
	* @param { Array<(string | number | BN)> } amounts,
	* @param { ArgumentTypes.AccountId } to,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"removeLiquidityByAmounts" (
		maxShareAmount: (string | number | BN),
		amounts: Array<(string | number | BN)>,
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::removeLiquidityByAmounts", [maxShareAmount, amounts, to], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* forceUpdateRates
	*
	* @returns { Result<null, ReturnTypes.LangError> }
	*/
	"forceUpdateRates" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<null, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::forceUpdateRates", [], __options , (result) => { return handleReturnType(result, getTypeDescription(28, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapExactIn
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } tokenInAmount,
	* @param { (string | number | BN) } minTokenOutAmount,
	* @param { ArgumentTypes.AccountId } to,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"swapExactIn" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		tokenInAmount: (string | number | BN),
		minTokenOutAmount: (string | number | BN),
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::swapExactIn", [tokenIn, tokenOut, tokenInAmount, minTokenOutAmount, to], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapExactOut
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } tokenOutAmount,
	* @param { (string | number | BN) } maxTokenInAmount,
	* @param { ArgumentTypes.AccountId } to,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"swapExactOut" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		tokenOutAmount: (string | number | BN),
		maxTokenInAmount: (string | number | BN),
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::swapExactOut", [tokenIn, tokenOut, tokenOutAmount, maxTokenInAmount, to], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* swapReceived
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } minTokenOutAmount,
	* @param { ArgumentTypes.AccountId } to,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"swapReceived" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		minTokenOutAmount: (string | number | BN),
		to: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::swapReceived", [tokenIn, tokenOut, minTokenOutAmount, to], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* setFeeReceiver
	*
	* @param { ArgumentTypes.AccountId | null } feeReceiver,
	* @returns { Result<Result<null, ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"setFeeReceiver" (
		feeReceiver: ArgumentTypes.AccountId | null,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::setFeeReceiver", [feeReceiver], __options , (result) => { return handleReturnType(result, getTypeDescription(13, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* setFees
	*
	* @param { (number | string | BN) } tradeFee,
	* @param { (number | string | BN) } protocolFee,
	* @returns { Result<Result<null, ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"setFees" (
		tradeFee: (number | string | BN),
		protocolFee: (number | string | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::setFees", [tradeFee, protocolFee], __options , (result) => { return handleReturnType(result, getTypeDescription(13, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* setAmpCoef
	*
	* @param { (string | number | BN) } ampCoef,
	* @returns { Result<Result<null, ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"setAmpCoef" (
		ampCoef: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::setAmpCoef", [ampCoef], __options , (result) => { return handleReturnType(result, getTypeDescription(13, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* tokens
	*
	* @returns { Result<Array<ReturnTypes.AccountId>, ReturnTypes.LangError> }
	*/
	"tokens" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Array<ReturnTypes.AccountId>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::tokens", [], __options , (result) => { return handleReturnType(result, getTypeDescription(29, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* reserves
	*
	* @returns { Result<Array<ReturnNumber>, ReturnTypes.LangError> }
	*/
	"reserves" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Array<ReturnNumber>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::reserves", [], __options , (result) => { return handleReturnType(result, getTypeDescription(30, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* ampCoef
	*
	* @returns { Result<ReturnNumber, ReturnTypes.LangError> }
	*/
	"ampCoef" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<ReturnNumber, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::ampCoef", [], __options , (result) => { return handleReturnType(result, getTypeDescription(31, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* fees
	*
	* @returns { Result<[number, number], ReturnTypes.LangError> }
	*/
	"fees" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<[number, number], ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::fees", [], __options , (result) => { return handleReturnType(result, getTypeDescription(32, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* feeReceiver
	*
	* @returns { Result<ReturnTypes.AccountId | null, ReturnTypes.LangError> }
	*/
	"feeReceiver" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<ReturnTypes.AccountId | null, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::feeReceiver", [], __options , (result) => { return handleReturnType(result, getTypeDescription(34, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* tokenRates
	*
	* @returns { Result<Array<ReturnNumber>, ReturnTypes.LangError> }
	*/
	"tokenRates" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Array<ReturnNumber>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::tokenRates", [], __options , (result) => { return handleReturnType(result, getTypeDescription(30, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getSwapAmountOut
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } tokenInAmount,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"getSwapAmountOut" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		tokenInAmount: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::getSwapAmountOut", [tokenIn, tokenOut, tokenInAmount], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getSwapAmountIn
	*
	* @param { ArgumentTypes.AccountId } tokenIn,
	* @param { ArgumentTypes.AccountId } tokenOut,
	* @param { (string | number | BN) } tokenOutAmount,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"getSwapAmountIn" (
		tokenIn: ArgumentTypes.AccountId,
		tokenOut: ArgumentTypes.AccountId,
		tokenOutAmount: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::getSwapAmountIn", [tokenIn, tokenOut, tokenOutAmount], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getMintLiquidityForAmounts
	*
	* @param { Array<(string | number | BN)> } amounts,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"getMintLiquidityForAmounts" (
		amounts: Array<(string | number | BN)>,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::getMintLiquidityForAmounts", [amounts], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getAmountsForLiquidityMint
	*
	* @param { (string | number | BN) } liquidity,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"getAmountsForLiquidityMint" (
		liquidity: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::getAmountsForLiquidityMint", [liquidity], __options , (result) => { return handleReturnType(result, getTypeDescription(26, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getBurnLiquidityForAmounts
	*
	* @param { Array<(string | number | BN)> } amounts,
	* @returns { Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"getBurnLiquidityForAmounts" (
		amounts: Array<(string | number | BN)>,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<[ReturnNumber, ReturnNumber], ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::getBurnLiquidityForAmounts", [amounts], __options , (result) => { return handleReturnType(result, getTypeDescription(23, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getAmountsForLiquidityBurn
	*
	* @param { (string | number | BN) } liquidity,
	* @returns { Result<Result<Array<ReturnNumber>, ReturnTypes.StablePoolError>, ReturnTypes.LangError> }
	*/
	"getAmountsForLiquidityBurn" (
		liquidity: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<Array<ReturnNumber>, ReturnTypes.StablePoolError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "stablePool::getAmountsForLiquidityBurn", [liquidity], __options , (result) => { return handleReturnType(result, getTypeDescription(26, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* totalSupply
	*
	* @returns { Result<ReturnNumber, ReturnTypes.LangError> }
	*/
	"totalSupply" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<ReturnNumber, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22::totalSupply", [], __options , (result) => { return handleReturnType(result, getTypeDescription(31, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* balanceOf
	*
	* @param { ArgumentTypes.AccountId } owner,
	* @returns { Result<ReturnNumber, ReturnTypes.LangError> }
	*/
	"balanceOf" (
		owner: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<ReturnNumber, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22::balanceOf", [owner], __options , (result) => { return handleReturnType(result, getTypeDescription(31, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* allowance
	*
	* @param { ArgumentTypes.AccountId } owner,
	* @param { ArgumentTypes.AccountId } spender,
	* @returns { Result<ReturnNumber, ReturnTypes.LangError> }
	*/
	"allowance" (
		owner: ArgumentTypes.AccountId,
		spender: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<ReturnNumber, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22::allowance", [owner, spender], __options , (result) => { return handleReturnType(result, getTypeDescription(31, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* transfer
	*
	* @param { ArgumentTypes.AccountId } to,
	* @param { (string | number | BN) } value,
	* @param { Array<(number | string | BN)> } data,
	* @returns { Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> }
	*/
	"transfer" (
		to: ArgumentTypes.AccountId,
		value: (string | number | BN),
		data: Array<(number | string | BN)>,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22::transfer", [to, value, data], __options , (result) => { return handleReturnType(result, getTypeDescription(35, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* transferFrom
	*
	* @param { ArgumentTypes.AccountId } from,
	* @param { ArgumentTypes.AccountId } to,
	* @param { (string | number | BN) } value,
	* @param { Array<(number | string | BN)> } data,
	* @returns { Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> }
	*/
	"transferFrom" (
		from: ArgumentTypes.AccountId,
		to: ArgumentTypes.AccountId,
		value: (string | number | BN),
		data: Array<(number | string | BN)>,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22::transferFrom", [from, to, value, data], __options , (result) => { return handleReturnType(result, getTypeDescription(35, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* approve
	*
	* @param { ArgumentTypes.AccountId } spender,
	* @param { (string | number | BN) } value,
	* @returns { Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> }
	*/
	"approve" (
		spender: ArgumentTypes.AccountId,
		value: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22::approve", [spender, value], __options , (result) => { return handleReturnType(result, getTypeDescription(35, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* increaseAllowance
	*
	* @param { ArgumentTypes.AccountId } spender,
	* @param { (string | number | BN) } deltaValue,
	* @returns { Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> }
	*/
	"increaseAllowance" (
		spender: ArgumentTypes.AccountId,
		deltaValue: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22::increaseAllowance", [spender, deltaValue], __options , (result) => { return handleReturnType(result, getTypeDescription(35, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* decreaseAllowance
	*
	* @param { ArgumentTypes.AccountId } spender,
	* @param { (string | number | BN) } deltaValue,
	* @returns { Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> }
	*/
	"decreaseAllowance" (
		spender: ArgumentTypes.AccountId,
		deltaValue: (string | number | BN),
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.PSP22Error>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22::decreaseAllowance", [spender, deltaValue], __options , (result) => { return handleReturnType(result, getTypeDescription(35, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* tokenName
	*
	* @returns { Result<string | null, ReturnTypes.LangError> }
	*/
	"tokenName" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<string | null, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22Metadata::tokenName", [], __options , (result) => { return handleReturnType(result, getTypeDescription(37, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* tokenSymbol
	*
	* @returns { Result<string | null, ReturnTypes.LangError> }
	*/
	"tokenSymbol" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<string | null, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22Metadata::tokenSymbol", [], __options , (result) => { return handleReturnType(result, getTypeDescription(37, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* tokenDecimals
	*
	* @returns { Result<number, ReturnTypes.LangError> }
	*/
	"tokenDecimals" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<number, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "psp22Metadata::tokenDecimals", [], __options , (result) => { return handleReturnType(result, getTypeDescription(39, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getOwner
	*
	* @returns { Result<Result<ReturnTypes.AccountId, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> }
	*/
	"getOwner" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<ReturnTypes.AccountId, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "ownable2Step::getOwner", [], __options , (result) => { return handleReturnType(result, getTypeDescription(40, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* getPendingOwner
	*
	* @returns { Result<Result<ReturnTypes.AccountId, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> }
	*/
	"getPendingOwner" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<ReturnTypes.AccountId, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "ownable2Step::getPendingOwner", [], __options , (result) => { return handleReturnType(result, getTypeDescription(40, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* transferOwnership
	*
	* @param { ArgumentTypes.AccountId } newOwner,
	* @returns { Result<Result<null, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> }
	*/
	"transferOwnership" (
		newOwner: ArgumentTypes.AccountId,
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "ownable2Step::transferOwnership", [newOwner], __options , (result) => { return handleReturnType(result, getTypeDescription(42, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* acceptOwnership
	*
	* @returns { Result<Result<null, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> }
	*/
	"acceptOwnership" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "ownable2Step::acceptOwnership", [], __options , (result) => { return handleReturnType(result, getTypeDescription(42, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* renounceOwnership
	*
	* @returns { Result<Result<null, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> }
	*/
	"renounceOwnership" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "ownable2Step::renounceOwnership", [], __options , (result) => { return handleReturnType(result, getTypeDescription(42, DATA_TYPE_DESCRIPTIONS)); });
	}

	/**
	* ensureOwner
	*
	* @returns { Result<Result<null, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> }
	*/
	"ensureOwner" (
		__options ? : GasLimit,
	): Promise< QueryReturnType< Result<Result<null, ReturnTypes.Ownable2StepError>, ReturnTypes.LangError> > >{
		return queryOkJSON( this.__apiPromise, this.__nativeContract, this.__callerAddress, "ownable2Step::ensureOwner", [], __options , (result) => { return handleReturnType(result, getTypeDescription(42, DATA_TYPE_DESCRIPTIONS)); });
	}

}