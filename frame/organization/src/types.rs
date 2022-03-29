use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Organization<AccountId, BlockNumber, BoundedString, BoundedProperty> {
	/// Organization ID
	pub id: AccountId,

	/// object name
	pub name: BoundedString,

	/// Description about the organization.
	pub description: BoundedString,

	/// admin of the object
	pub admin: AccountId,

	/// Official website url
	pub website: BoundedString,

	/// Official email address
	pub email: BoundedString,

	/// Whether the organization suspended or not
	pub suspended: bool,

	/// Created at block
	pub block: BlockNumber,

	/// Creation timestamp
	pub timestamp: u64,

	/// Custom properties
	pub props: Option<BoundedProperty>,
}
