import type * as EventTypes from '../event-types/stable_pool_contract';
import type {ContractPromise} from "@polkadot/api-contract";
import type {ApiPromise} from "@polkadot/api";
import EVENT_DATA_TYPE_DESCRIPTIONS from '../event-data/stable_pool_contract.json';
import {getEventTypeDescription} from "../shared/utils";
import {handleEventReturn} from "@727-ventures/typechain-types";

export default class EventsClass {
	readonly __nativeContract : ContractPromise;
	readonly __api : ApiPromise;

	constructor(
		nativeContract : ContractPromise,
		api : ApiPromise,
	) {
		this.__nativeContract = nativeContract;
		this.__api = api;
	}

	public subscribeOnAddLiquidityEvent(callback : (event : EventTypes.AddLiquidity) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('AddLiquidity', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.AddLiquidity);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'AddLiquidity');
	}

	public subscribeOnRemoveLiquidityEvent(callback : (event : EventTypes.RemoveLiquidity) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('RemoveLiquidity', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.RemoveLiquidity);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'RemoveLiquidity');
	}

	public subscribeOnSwapEvent(callback : (event : EventTypes.Swap) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('Swap', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.Swap);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'Swap');
	}

	public subscribeOnSyncEvent(callback : (event : EventTypes.Sync) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('Sync', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.Sync);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'Sync');
	}

	public subscribeOnRatesUpdatedEvent(callback : (event : EventTypes.RatesUpdated) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('RatesUpdated', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.RatesUpdated);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'RatesUpdated');
	}

	public subscribeOnApprovalEvent(callback : (event : EventTypes.Approval) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('Approval', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.Approval);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'Approval');
	}

	public subscribeOnTransferEvent(callback : (event : EventTypes.Transfer) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('Transfer', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.Transfer);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'Transfer');
	}

	public subscribeOnTransferOwnershipInitiatedEvent(callback : (event : EventTypes.TransferOwnershipInitiated) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('TransferOwnershipInitiated', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.TransferOwnershipInitiated);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'TransferOwnershipInitiated');
	}

	public subscribeOnTransferOwnershipAcceptedEvent(callback : (event : EventTypes.TransferOwnershipAccepted) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('TransferOwnershipAccepted', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.TransferOwnershipAccepted);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'TransferOwnershipAccepted');
	}

	public subscribeOnOwnershipRenouncedEvent(callback : (event : EventTypes.OwnershipRenounced) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('OwnershipRenounced', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.OwnershipRenounced);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'OwnershipRenounced');
	}

	public subscribeOnFeeReceiverChangedEvent(callback : (event : EventTypes.FeeReceiverChanged) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('FeeReceiverChanged', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.FeeReceiverChanged);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'FeeReceiverChanged');
	}

	public subscribeOnAmpCoefChangedEvent(callback : (event : EventTypes.AmpCoefChanged) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('AmpCoefChanged', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.AmpCoefChanged);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'AmpCoefChanged');
	}

	public subscribeOnFeeChangedEvent(callback : (event : EventTypes.FeeChanged) => void) {
		const callbackWrapper = (args: any[], event: any) => {
			const _event: Record < string, any > = {};

			for (let i = 0; i < args.length; i++) {
				_event[event.args[i]!.name] = args[i]!.toJSON();
			}

			callback(handleEventReturn(_event, getEventTypeDescription('FeeChanged', EVENT_DATA_TYPE_DESCRIPTIONS)) as EventTypes.FeeChanged);
		};

		return this.__subscribeOnEvent(callbackWrapper, (eventName : string) => eventName == 'FeeChanged');
	}


	private __subscribeOnEvent(
		callback : (args: any[], event: any) => void,
		filter : (eventName: string) => boolean = () => true
	) {
		// @ts-ignore
		return this.__api.query.system.events((events) => {
			events.forEach((record: any) => {
				const { event } = record;

				if (event.method == 'ContractEmitted') {
					const [address, data] = record.event.data;

					if (address.toString() === this.__nativeContract.address.toString()) {
						const {args, event} = this.__nativeContract.abi.decodeEvent(data);

						if (filter(event.identifier.toString()))
							callback(args, event);
					}
				}
			});
		});
	}

}