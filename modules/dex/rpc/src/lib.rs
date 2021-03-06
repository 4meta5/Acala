//! RPC interface for the dex module.

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use module_dex_rpc_runtime_api::BalanceInfo;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, MaybeDisplay, MaybeFromStr},
};
use std::sync::Arc;

pub use self::gen_client::Client as DexClient;
pub use module_dex_rpc_runtime_api::DexApi as DexRuntimeApi;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct BalanceRequest<Balance> {
	amount: Balance,
}

#[rpc]
pub trait DexApi<BlockHash, CurrencyId, Balance, ResponseType> {
	#[rpc(name = "dex_getSupplyAmount")]
	fn get_supply_amount(
		&self,
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		target_currency_amount: BalanceRequest<Balance>,
		at: Option<BlockHash>,
	) -> Result<ResponseType>;

	#[rpc(name = "dex_getTargetAmount")]
	fn get_target_amount(
		&self,
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		supply_currency_amount: BalanceRequest<Balance>,
		at: Option<BlockHash>,
	) -> Result<ResponseType>;
}

/// A struct that implements the [`DexApi`].
pub struct Dex<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> Dex<C, B> {
	/// Create new `Dex` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Dex {
			client,
			_marker: Default::default(),
		}
	}
}

pub enum Error {
	/// The call to runtime failed.
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

impl<C, Block, CurrencyId, Balance> DexApi<<Block as BlockT>::Hash, CurrencyId, Balance, BalanceInfo<Balance>>
	for Dex<C, Block>
where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: DexRuntimeApi<Block, CurrencyId, Balance>,
	CurrencyId: Codec,
	Balance: Codec + MaybeDisplay + MaybeFromStr,
{
	fn get_supply_amount(
		&self,
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		target_currency_amount: BalanceRequest<Balance>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));
		let BalanceRequest { amount } = target_currency_amount;

		api.get_supply_amount(&at, supply_currency_id, target_currency_id, amount)
			.map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to get supply amount.".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}

	fn get_target_amount(
		&self,
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		supply_currency_amount: BalanceRequest<Balance>,
		at: Option<<Block as BlockT>::Hash>,
	) -> Result<BalanceInfo<Balance>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or_else(||
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash));
		let BalanceRequest { amount } = supply_currency_amount;

		api.get_target_amount(&at, supply_currency_id, target_currency_id, amount)
			.map_err(|e| RpcError {
				code: ErrorCode::ServerError(Error::RuntimeError.into()),
				message: "Unable to get target amount.".into(),
				data: Some(format!("{:?}", e).into()),
			})
	}
}
