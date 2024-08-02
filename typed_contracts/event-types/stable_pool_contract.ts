import type {ReturnNumber} from "@727-ventures/typechain-types";
import type * as ReturnTypes from '../types-returns/stable_pool_contract';

export interface AddLiquidity {
	provider: ReturnTypes.AccountId;
	tokenAmounts: Array<ReturnNumber>;
	shares: ReturnNumber;
	to: ReturnTypes.AccountId;
}

export interface RemoveLiquidity {
	provider: ReturnTypes.AccountId;
	tokenAmounts: Array<ReturnNumber>;
	shares: ReturnNumber;
	to: ReturnTypes.AccountId;
}

export interface Swap {
	sender: ReturnTypes.AccountId;
	tokenIn: ReturnTypes.AccountId;
	amountIn: ReturnNumber;
	tokenOut: ReturnTypes.AccountId;
	amountOut: ReturnNumber;
	to: ReturnTypes.AccountId;
}

export interface Sync {
	reserves: Array<ReturnNumber>;
}

export interface RatesUpdated {
	rates: Array<ReturnTypes.TokenRate>;
}

export interface Approval {
	owner: ReturnTypes.AccountId;
	spender: ReturnTypes.AccountId;
	amount: ReturnNumber;
}

export interface Transfer {
	from: ReturnTypes.AccountId | null;
	to: ReturnTypes.AccountId | null;
	value: ReturnNumber;
}

export interface TransferOwnershipInitiated {
	newOwner: ReturnTypes.AccountId;
}

export interface TransferOwnershipAccepted {
	newOwner: ReturnTypes.AccountId;
}

export interface OwnershipRenounced {

}

export interface FeeReceiverChanged {
	newFeeReceiver: ReturnTypes.AccountId | null;
}

export interface AmpCoefChanged {
	newAmpCoef: ReturnNumber;
}

export interface FeeChanged {
	tradeFee: number;
	protocolFee: number;
}

