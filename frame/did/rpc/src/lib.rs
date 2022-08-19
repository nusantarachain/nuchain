use codec::Codec;
use jsonrpsee::{
	core::{Error as JsonRpseeError, RpcResult},
	proc_macros::rpc,
};
use sc_client_api::{BlockBackend, HeaderBackend};
use sc_rpc_api::DenyUnsafe;
use sp_api::{ProvideRuntimeApi, BlockId};
use sp_runtime::traits::Block as BlockT;
use std::{
	marker::{PhantomData, Send, Sync},
	sync::Arc,
};

#[rpc(client, server)]
pub trait DidApi<BlockHash, AccountId> {
	/// Get owner of the did object, given a id `AccountId`
	/// this returns:
	/// owner of the object id `AccountId`.
	#[method(name = "did_getOwner")]
	fn get_owner(&self, id: AccountId) -> RpcResult<Option<AccountId>>;
}

pub struct Did<Block: BlockT, Client> {
	client: Arc<Client>,
	deny_unsafe: DenyUnsafe,
	_marker: PhantomData<Block>,
}

impl<Block: BlockT, Client> Did<Block, Client> {
	/// Create a new Did API.
	pub fn new(client: Arc<Client>, deny_unsafe: DenyUnsafe) -> Self {
		Self { client, deny_unsafe, _marker: PhantomData::default() }
	}
}

pub use pallet_did_runtime_api::DidApi as DidRuntimeApi;

impl<Block, Client, AccountId> 
    DidApiServer<Block::Hash, AccountId> 
    for Did<Block, Client>
where
	Block: BlockT,
	Client: BlockBackend<Block>
		+ HeaderBackend<Block>
		+ ProvideRuntimeApi<Block>
		+ Send
		+ Sync
		+ 'static,
    AccountId: Codec + Send + Sync + Clone,
    Client::Api: pallet_did_runtime_api::DidApi<Block, AccountId>,
{
	fn get_owner(&self, id: AccountId) -> RpcResult<Option<AccountId>> {
		self.deny_unsafe.check_if_safe()?;
		let api = self.client.runtime_api();
		let block_id = BlockId::hash(self.client.info().best_hash);

		match api.get_owner(&block_id, id.clone()){
            Err(e) => Err(JsonRpseeError::to_call_error(e)),
            Ok(None) => Ok(Some(id)), // just return the entered AccountId if no owner is found
            Ok(r) => Ok(r),
        }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		let result = add(2, 2);
		assert_eq!(result, 4);
	}
}
