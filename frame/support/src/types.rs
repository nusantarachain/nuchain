
use codec::{Decode, Encode};
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;
use scale_info::TypeInfo;

pub type Text = Vec<u8>;
pub type PropName = Vec<u8>;
pub type PropValue = Vec<u8>;


// Contains a name-value pair for a product property e.g. description: Ingredient ABC
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Property {
    // Name of the product property e.g. desc or description
    name: PropName,
    // Value of the product property e.g. Ingredient ABC
    value: PropValue,
}

impl Property {
    pub fn new(name: &[u8], value: &[u8]) -> Self {
        Self {
            name: name.to_vec(),
            value: value.to_vec(),
        }
    }

    pub fn name(&self) -> &[u8] {
        self.name.as_ref()
    }

    pub fn value(&self) -> &[u8] {
        self.value.as_ref()
    }
}


