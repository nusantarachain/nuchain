
pub use scale_info::TypeInfo;


/// Error generated when validating a DID operation.
#[derive(Debug, Eq, PartialEq, TypeInfo)]
pub enum SignatureError {
	/// The signature is not in the format the verification key expects.
	InvalidSignatureFormat,
	/// The signature is invalid for the payload and the verification key
	/// provided.
	InvalidSignature,
	/// The operation nonce is not equal to the current DID nonce + 1.
	InvalidNonce,
	/// The provided operation block number is not valid.
	TransactionExpired,
}


