// this is included file to handle struct's fields access easily.

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct AllowedMintAccount<AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq> {
    account: AccountId,
    amount: u32,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct NewCollectionParam<
    Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
    AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
    /// Instance name
    name: Vec<u8>,

    /// Code symbol of the asset
    symbol: Vec<u8>,

    /// Can change `owner`, `issuer`, `freezer` and `admin` accounts.
    owner: AccountId,

    /// Max supply of unique token that will be appeared in this collection.
    max_asset_count: u32,

    /// Whether is token available for this collection.
    has_token: bool,

    /// Max token supply when `has_token` is true.
    max_token_supply: Balance,

    /// The ED for virtual accounts.
    min_balance: Balance,

    // // /// Can mint tokens.
    // // issuer: AccountId,

    // /// Whether only eligible account cant mint
    // allowed_mint_only: bool,
    /// anyone from public origin can mint tokens.
    public_mintable: bool,

    /// List of allowed accounts to mint if `public_mintable` == false.
    allowed_mint_accounts: Vec<AllowedMintAccount<AccountId>>,

    // /// Can thaw tokens, force transfers and burn tokens from any account.
    // admin: AccountId,
    // /// Can freeze tokens.
    // freezer: AccountId,
    /// The total circulating supply across all accounts.
    // supply: u32,

    /// Max limit holding token per account.
    max_asset_per_account: u32,

    /// The number of balance-holding accounts that this asset may have, excluding those that were
    /// created when they had a system-level ED.
    max_zombies: u32,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug)]
pub struct CollectionMetadata<
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

    /// Max count of asset that will be appeared in this collection.
    max_asset_count: u32,

    /// Whether is token available for this collection.
    has_token: bool,

    /// Max token supply when `has_token` is true.
    max_token_supply: Balance,

    // /// Whether only eligible account cant mint
    // allowed_mint_only: bool,
    /// anyone from public origin can mint tokens.
    public_mintable: bool,

    /// The ED for virtual accounts.
    min_balance: Balance,

    // /// Can thaw tokens, force transfers and burn tokens from any account.
    // admin: AccountId,
    // /// Can freeze tokens.
    // freezer: AccountId,
    /// The total asset across all accounts.
    asset_count: u64,

    /// Incremental asset index
    asset_index: u64,

    /// The total available supply of sub-token
    token_supply: Balance,

    /// The balance deposited for this struct.
    ///
    /// This pays for the data stored here together with any virtual accounts.
    deposit: DepositBalance,

    /// The number of balance-holding accounts that this asset may have, excluding those that were
    /// created when they had a system-level ED.
    max_zombies: u32,

    /// The current number of zombie accounts.
    zombies: u32,

    /// The total number of accounts.
    accounts: u32,

    /// Whether the asset is frozen for permissionless transfers.
    is_frozen: bool,

    /// Max limit holding token per account.
    max_asset_per_account: u32,
}


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct AssetMetadata<
    DepositBalance: Encode + Decode + Clone + Debug + Eq + PartialEq,
    Balance: Encode + Decode + Clone + Debug + Eq + PartialEq,
    AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq,
> {
    /// The user friendly name of this asset. Limited in length by `StringLimit`.
    name: Vec<u8>,

    /// Description of this asset. Limited in length by `StringLimit`.
    description: Vec<u8>,

    // /// The number of decimals this asset uses to represent one unit.
    // decimals: u8,

    /// Asset's image URI
    image_uri: Vec<u8>,

    /// Base URI
    /// based on https://docs.openzeppelin.com/contracts/2.x/api/token/erc721#ERC721Metadata
    base_uri: Vec<u8>,

    /// Intelectual property owner
    ip_owner: AccountId,

    /// The balance deposited for this metadata.
    ///
    /// This pays for the data stored in this struct.
    deposit: DepositBalance,

    /// Available supply for this asset's token (sub-token).
    token_supply: Balance
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct AssetOwnership<
    AccountId: Encode + Decode + Clone + Debug + Eq + PartialEq
> {
    /// Asset owner
    owner: AccountId,

    /// Other account that allowed/approved to transfer this asset
    approved_to_transfer: Option<AccountId>,
    
    /// Other account that allowed/approved to transfer this asset's token
    approved_to_transfer_token: Option<AccountId>,

    /// List of this asset's token holder, see `TokenBalance` to get balances
    /// for each holders
    token_holders: Vec<AccountId>,
}

#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default)]
pub struct TokenBalance<Balance: Encode + Decode + Clone + Debug + Eq + PartialEq> {
    /// The balance.
    balance: Balance,
    /// Whether the account is frozen.
    is_frozen: bool,
    /// Whether the account is a zombie. If not, then it has a reference.
    is_zombie: bool,
}
