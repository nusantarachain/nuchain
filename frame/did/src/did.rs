use crate::types::AttributedId;

use frame_support::dispatch::DispatchResult;

pub trait Did<AccountId, BlockNumber, Moment, Signature, BoundedString> {
    fn is_owner(identity: &AccountId, actual_owner: &AccountId) -> DispatchResult;
    fn identity_owner(identity: &AccountId) -> AccountId;
    fn valid_delegate(
        identity: &AccountId,
        delegate_type: &Vec<u8>,
        delegate: &AccountId,
    ) -> DispatchResult;
    fn valid_listed_delegate(
        identity: &AccountId,
        delegate_type: &Vec<u8>,
        delegate: &AccountId,
    ) -> DispatchResult;
    fn create_delegate(
        who: &AccountId,
        identity: &AccountId,
        delegate: &AccountId,
        delegate_type: &Vec<u8>,
        valid_for: Option<BlockNumber>,
    ) -> DispatchResult;
    fn check_signature(signature: &Signature, msg: &Vec<u8>, signer: &AccountId) -> DispatchResult;
    fn valid_signer(
        identity: &AccountId,
        signature: &Signature,
        msg: &Vec<u8>,
        signer: &AccountId,
    ) -> DispatchResult;
    fn create_attribute(
        who: &AccountId,
        identity: &AccountId,
        name: &Vec<u8>,
        value: &Vec<u8>,
        valid_for: Option<BlockNumber>,
    ) -> DispatchResult;
    fn reset_attribute(who: AccountId, identity: &AccountId, name: &BoundedString) -> DispatchResult;
    fn valid_attribute(identity: &AccountId, name: &BoundedString, value: &BoundedString) -> DispatchResult;
    fn attribute_and_id(
        identity: &AccountId,
        name: &BoundedString,
    ) -> Option<AttributedId<BlockNumber, BoundedString>>;
}
