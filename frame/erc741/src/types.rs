// this is included file to handle struct's fields access easily.

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct ERC721Details<
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
	DepositBalance: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
    /// Instance name
    name: Vec<u8>,

    /// Code symbol of the asset
    symbol: Vec<u8>,

	/// Can change `owner`, `issuer`, `freezer` and `admin` accounts.
	owner: AccountId,
	// /// Can mint tokens.
	// issuer: AccountId,

    /// Whether only eligible account cant mint
    eligible_mint_only: bool,

    /// eligible accounts to mint
    /// list of accounts that eligible to mint new token
    /// this variable is used only when eligible_mint_only is true
    eligible_mint_accounts: Vec<AccountId>,

	/// Can thaw tokens, force transfers and burn tokens from any account.
	admin: AccountId,
	/// Can freeze tokens.
	freezer: AccountId,
	/// The total supply across all accounts.
	supply: Balance,
	/// The balance deposited for this struct.
	///
	/// This pays for the data stored here together with any virtual accounts.
	deposit: DepositBalance,
	// /// The number of balance-holding accounts that this asset may have, excluding those that were
	// /// created when they had a system-level ED.
	// max_zombies: u32,
	// /// The ED for virtual accounts.
	// min_balance: Balance,
	// /// The current number of zombie accounts.
	// zombies: u32,
	/// The total number of accounts.
	accounts: u32,
	/// Whether the asset is frozen for permissionless transfers.
	is_frozen: bool,

    /// Max limit holding token per account.
    max_token_per_account: u32
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct ERC20Details<
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
	AssetId: Encode + Decode + Clone + Debug + Eq + PartialEq,
	DepositBalance: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
    /// ERC721 ID where this ERC20 linked to
    asset_id: AssetId,
	/// Can change `owner`, `issuer`, `freezer` and `admin` accounts.
	owner: AccountId,
	// /// Can mint tokens.
	// issuer: AccountId,
    
	// /// Can thaw tokens, force transfers and burn tokens from any account.
	// admin: AccountId,
	// /// Can freeze tokens.
	// freezer: AccountId,
	/// The total supply across all accounts.
	supply: Balance,
	/// The balance deposited for this asset.
	///
	/// This pays for the data stored here together with any virtual accounts.
	deposit: DepositBalance,
	/// The number of balance-holding accounts that this asset may have, excluding those that were
	/// created when they had a system-level ED.
	max_zombies: u32,
	/// The ED for virtual accounts.
	min_balance: Balance,
	/// The current number of zombie accounts.
	zombies: u32,
	/// The total number of accounts.
	accounts: u32,
	/// Whether the asset is frozen for permissionless transfers.
	is_frozen: bool,
}


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct TokenBalance<
	Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
	/// The balance.
	balance: Balance,
	/// Whether the account is frozen.
	is_frozen: bool,
	/// Whether the account is a zombie. If not, then it has a reference.
	is_zombie: bool,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct AssetMetadata<DepositBalance> {
	/// The balance deposited for this metadata.
	///
	/// This pays for the data stored in this struct.
	deposit: DepositBalance,
	/// The user friendly name of this asset. Limited in length by `StringLimit`.
	name: Vec<u8>,
	/// The ticker symbol for this asset. Limited in length by `StringLimit`.
	symbol: Vec<u8>,
	// /// The number of decimals this asset uses to represent one unit.
	// decimals: u8,

    /// Token URI
    token_uri:Vec<u8>,

    /// Base URI
    /// based on https://docs.openzeppelin.com/contracts/2.x/api/token/erc721#ERC721Metadata
    base_uri:Vec<u8>,
}

