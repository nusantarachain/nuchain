use codec::{Decode, Encode};
use frame_support::{
	pallet_prelude::{BoundedVec, MaxEncodedLen},
	traits::{ConstU32, Get},
};
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

/// Attributes or properties that make an identity.
#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug, scale_info::TypeInfo, MaxEncodedLen,
)]
pub struct Attribute<BlockNumber, BoundedString> {
	pub name: BoundedString,
	pub value: BoundedString,
	pub validity: BlockNumber,
	pub creation: u64,
	pub nonce: u64,
}

pub type AttributedId<BlockNumber, BoundedString> = (Attribute<BlockNumber, BoundedString>, [u8; 32]);

/// Off-chain signed transaction.
#[derive(
	PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode, Default, RuntimeDebug, scale_info::TypeInfo, MaxEncodedLen,
)]
pub struct AttributeTransaction<Signature, AccountId, BoundedString> {
	pub signature: Signature,
	pub name: BoundedString,
	pub value: BoundedString,
	pub validity: u32,
	pub signer: AccountId,
	pub identity: AccountId,
}
