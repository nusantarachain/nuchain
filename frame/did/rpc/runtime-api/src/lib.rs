#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;

sp_api::decl_runtime_apis! {
	pub trait DidApi<AccountId> 
    where 
        AccountId: Codec + Send + Sync,
    {
		/// Get owner of the did object, given a id `AccountId`
		/// this returns:
		/// owner of the object id `AccountId`.
		fn get_owner(id: AccountId) -> Option<AccountId>;
	}
}
