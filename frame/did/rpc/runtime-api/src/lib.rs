#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode, MaxEncodedLen};


// #[derive(Encode, Decode, Clone, PartialEq, Eq)]
// pub struct DidService {

// }

// #[derive(Encode, Decode, TypeInfo, Eq, PartialEq)]
// pub struct DidDocument {
//     pub id: Vec<u8>,
//     pub controller: Vec<u8>,
//     pub verification_method: Vec<u8>,
//     pub authentication: Vec<u8>,
//     // pub assertion_method: Vec<u8>,
//     // pub key_agreement: Vec<u8>,
//     // pub capability_invocation: Vec<u8>,
//     // pub capability_delegation: Vec<u8>,
//     pub service: Vec<DidService>,
//     pub created: Vec<u8>,
//     pub updated: Vec<u8>,
// }

sp_api::decl_runtime_apis! {
	pub trait DidApi<AccountId> 
    where 
        AccountId: Codec + Send + Sync,
    {
        // fn get_document(uri: DidIdentifier) -> DidDocument;

		/// Get owner of the did object, given a id `AccountId`
		/// this returns:
		/// owner of the object id `AccountId`.
		fn get_owner(id: AccountId) -> Option<AccountId>;
	}
}
