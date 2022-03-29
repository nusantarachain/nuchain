use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{RuntimeDebug, DispatchError};
use sp_std::vec::Vec;

use crate::BoundedVec;

pub type Text = Vec<u8>;
pub type PropName<LN> = BoundedVec<u8, LN>;
pub type PropValue<LN> = BoundedVec<u8, LN>;

// Contains a name-value pair for a product property e.g. description: Ingredient ABC
#[derive(Encode, Decode, Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct Property<NameT, ValueT> {
	// Name of the product property e.g. desc or description
	name: NameT,
	// Value of the product property e.g. Ingredient ABC
	value: ValueT,
}

impl<NameT, ValueT> Property<NameT, ValueT>
where
	NameT: AsRef<[u8]>,
	ValueT: AsRef<[u8]>,
{
	pub fn new(name: NameT, value: ValueT) -> Self {
		Self { name, value }
	}

	pub fn name(&self) -> &[u8] {
		self.name.as_ref()
	}

	pub fn value(&self) -> &[u8] {
		self.value.as_ref()
	}
}

impl<NameT, ValueT> From<Property<Text, Text>>
	for Property<BoundedVec<u8, NameT>, BoundedVec<u8, ValueT>>
where
	BoundedVec<u8, NameT>: TryFrom<Text>,
	BoundedVec<u8, ValueT>: TryFrom<Text>,
{
	fn from(prop: Property<Text, Text>) -> Self {
		Self {
			name: prop.name.clone().try_into().ok().unwrap_or_default(),
			value: prop.value.clone().try_into().ok().unwrap_or_default(),
		}
	}
}

