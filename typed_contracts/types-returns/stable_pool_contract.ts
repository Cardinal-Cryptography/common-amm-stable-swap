import type BN from 'bn.js';
import type {ReturnNumber} from '@727-ventures/typechain-types';

export type AccountId = string | number[]

export interface TokenRate {
	constant ? : ReturnNumber,
	external ? : ExternalTokenRate
}

export class TokenRateBuilder {
	static Constant(value: ReturnNumber): TokenRate {
		return {
			constant: value,
		};
	}
	static External(value: ExternalTokenRate): TokenRate {
		return {
			external: value,
		};
	}
}

export type ExternalTokenRate = {
	cachedTokenRate: ReturnNumber,
	lastTokenRateUpdateTs: number,
	tokenRateContract: AccountId,
	expirationDurationMs: number
}

export interface StablePoolError {
	ownable2StepError ? : Ownable2StepError,
	mathError ? : MathError,
	psp22Error ? : PSP22Error,
	langError ? : LangError,
	invalidTokenId ? : AccountId,
	identicalTokenId ? : null,
	incorrectAmountsCount ? : null,
	invalidAmpCoef ? : null,
	insufficientLiquidityMinted ? : null,
	insufficientLiquidityBurned ? : null,
	insufficientOutputAmount ? : null,
	insufficientLiquidity ? : null,
	insufficientInputAmount ? : null,
	incorrectTokenCount ? : null,
	tooLargeTokenDecimal ? : null,
	invalidFee ? : null
}

export class StablePoolErrorBuilder {
	static Ownable2StepError(value: Ownable2StepError): StablePoolError {
		return {
			ownable2StepError: value,
		};
	}
	static MathError(value: MathError): StablePoolError {
		return {
			mathError: value,
		};
	}
	static PSP22Error(value: PSP22Error): StablePoolError {
		return {
			psp22Error: value,
		};
	}
	static LangError(value: LangError): StablePoolError {
		return {
			langError: value,
		};
	}
	static InvalidTokenId(value: AccountId): StablePoolError {
		return {
			invalidTokenId: value,
		};
	}
	static IdenticalTokenId(): StablePoolError {
		return {
			identicalTokenId: null,
		};
	}
	static IncorrectAmountsCount(): StablePoolError {
		return {
			incorrectAmountsCount: null,
		};
	}
	static InvalidAmpCoef(): StablePoolError {
		return {
			invalidAmpCoef: null,
		};
	}
	static InsufficientLiquidityMinted(): StablePoolError {
		return {
			insufficientLiquidityMinted: null,
		};
	}
	static InsufficientLiquidityBurned(): StablePoolError {
		return {
			insufficientLiquidityBurned: null,
		};
	}
	static InsufficientOutputAmount(): StablePoolError {
		return {
			insufficientOutputAmount: null,
		};
	}
	static InsufficientLiquidity(): StablePoolError {
		return {
			insufficientLiquidity: null,
		};
	}
	static InsufficientInputAmount(): StablePoolError {
		return {
			insufficientInputAmount: null,
		};
	}
	static IncorrectTokenCount(): StablePoolError {
		return {
			incorrectTokenCount: null,
		};
	}
	static TooLargeTokenDecimal(): StablePoolError {
		return {
			tooLargeTokenDecimal: null,
		};
	}
	static InvalidFee(): StablePoolError {
		return {
			invalidFee: null,
		};
	}
}

export interface Ownable2StepError {
	callerNotOwner ? : AccountId,
	callerNotPendingOwner ? : AccountId,
	contractNotPendingOwner ? : AccountId,
	noPendingOwner ? : null
}

export class Ownable2StepErrorBuilder {
	static CallerNotOwner(value: AccountId): Ownable2StepError {
		return {
			callerNotOwner: value,
		};
	}
	static CallerNotPendingOwner(value: AccountId): Ownable2StepError {
		return {
			callerNotPendingOwner: value,
		};
	}
	static ContractNotPendingOwner(value: AccountId): Ownable2StepError {
		return {
			contractNotPendingOwner: value,
		};
	}
	static NoPendingOwner(): Ownable2StepError {
		return {
			noPendingOwner: null,
		};
	}
}

export interface MathError {
	addOverflow ? : number,
	castOverflow ? : number,
	divByZero ? : number,
	mulOverflow ? : number,
	subUnderflow ? : number,
	precision ? : number
}

export class MathErrorBuilder {
	static AddOverflow(value: number): MathError {
		return {
			addOverflow: value,
		};
	}
	static CastOverflow(value: number): MathError {
		return {
			castOverflow: value,
		};
	}
	static DivByZero(value: number): MathError {
		return {
			divByZero: value,
		};
	}
	static MulOverflow(value: number): MathError {
		return {
			mulOverflow: value,
		};
	}
	static SubUnderflow(value: number): MathError {
		return {
			subUnderflow: value,
		};
	}
	static Precision(value: number): MathError {
		return {
			precision: value,
		};
	}
}

export interface PSP22Error {
	custom ? : string,
	insufficientBalance ? : null,
	insufficientAllowance ? : null,
	zeroRecipientAddress ? : null,
	zeroSenderAddress ? : null,
	safeTransferCheckFailed ? : string
}

export class PSP22ErrorBuilder {
	static Custom(value: string): PSP22Error {
		return {
			custom: value,
		};
	}
	static InsufficientBalance(): PSP22Error {
		return {
			insufficientBalance: null,
		};
	}
	static InsufficientAllowance(): PSP22Error {
		return {
			insufficientAllowance: null,
		};
	}
	static ZeroRecipientAddress(): PSP22Error {
		return {
			zeroRecipientAddress: null,
		};
	}
	static ZeroSenderAddress(): PSP22Error {
		return {
			zeroSenderAddress: null,
		};
	}
	static SafeTransferCheckFailed(value: string): PSP22Error {
		return {
			safeTransferCheckFailed: value,
		};
	}
}

export enum LangError {
	couldNotReadInput = 'CouldNotReadInput'
}

