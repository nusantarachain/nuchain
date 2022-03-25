
use codec::{Decode, Encode, MaxEncodedLen};
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;
use scale_info::TypeInfo;

use crate::{BoundedVec, traits::Get};

pub type Text = Vec<u8>;
pub type PropName<LN> = BoundedVec<u8, LN>;
pub type PropValue<LN> = BoundedVec<u8, LN>;


// Contains a name-value pair for a product property e.g. description: Ingredient ABC
#[derive(Encode, Decode, Clone, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct Property<LN> where LN: Get<u32> {
    // Name of the product property e.g. desc or description
    name: PropName<LN>,
    // Value of the product property e.g. Ingredient ABC
    value: PropValue<LN>,
}

impl<LN> Property<LN> where LN: Get<u32> {
    pub fn new(name: PropName<LN>, value: PropValue<LN>) -> Self {
        Self {
            name: name,
            value: value,
        }
    }

    pub fn name(&self) -> &[u8] {
        self.name.as_ref()
    }

    pub fn value(&self) -> &[u8] {
        self.value.as_ref()
    }
}


