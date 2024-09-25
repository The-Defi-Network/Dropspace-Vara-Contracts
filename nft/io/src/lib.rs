#![no_std]
use gmeta::{In, InOut, Metadata};
use gstd::{prelude::*, ActorId};

pub type TokenId = u128;
pub const ZERO_ID: ActorId = ActorId::zero();

pub struct NftMetadata;

impl Metadata for NftMetadata {
    type Init = In<InitNft>;
    type Handle = InOut<NftAction, NftEvent>;
    type Reply = ();
    type Others = ();
    type Signal = ();
    type State = InOut<StateQuery, StateReply>;
}

#[derive(Default, Debug, Encode, Decode, TypeInfo)]
pub struct Config {
    pub supply_limit: u128,
    pub mint_price: u128,
    pub mint_fee: u128,
    pub mint_limit: u128,
    pub sale_time: u64,
    pub dev_wallet: ActorId,
    pub withdraw_wallet: ActorId,
}

#[derive(Default, Debug, Encode, Decode, TypeInfo)]
pub struct InitNft {
    pub collection: Collection,
    pub config: Config,
}

#[derive(Default, Debug, Encode, Decode, TypeInfo)]
pub struct Collection {
    pub name: String,
    pub description: String,
    pub symbol: String,
    pub base_uri: String,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NftAction {
    Buy {
        amount: u128,
    },
    Reserve {
        amount: u128,
    },
    /* Mint {
        to: ActorId,
        token_metadata: TokenMetadata,
    }, */
    Burn {
        token_id: TokenId,
    },
    Transfer {
        to: ActorId,
        token_id: TokenId,
    },
    Approve {
        to: ActorId,
        token_id: TokenId,
    },
    GetOwner {
        token_id: TokenId,
    },
    CheckIfApproved {
        to: ActorId,
        token_id: TokenId,
    },

    // Collection info
    SetName {
        name: String,
    },
    SetDescription {
        description: String,
    },
    SetSymbol {
        symbol: String,
    },
    SetBaseUri {
        base_uri: String,
    },

    // Collection configuration
    SetWithdrawWallet {
        withdraw_wallet: ActorId,
    },    
    SetSupplyLimit {
        supply_limit: u128,
    },
    SetMintLimit {
        mint_limit: u128,
    },
    SetMintPrice {
        mint_price: u128,
    },
    SetSaleTime {
        sale_time: u64,
    },
    ToggleSaleActive {},
}

#[derive(Debug, Encode, Decode, TypeInfo)]
pub enum NftEvent {
    Bought {
        to: ActorId,
        amount: u128,
    },
    Minted {
        to: ActorId,
        token_metadata: TokenMetadata,
    },
    Reserved {
        to: ActorId,
        amount: u128,
    },
    Burnt {
        token_id: TokenId,
    },
    Transferred {
        from: ActorId,
        to: ActorId,
        token_id: TokenId,
    },
    Approved {
        owner: ActorId,
        approved_account: ActorId,
        token_id: TokenId,
    },
    Owner {
        owner: ActorId,
        token_id: TokenId,
    },
    CheckIfApproved {
        to: ActorId,
        token_id: TokenId,
        approved: bool,
    },

    // Change Event for collection info
    NameChanged {
        name: String,
    },
    DescriptionChanged {
        description: String,
    },
    SymbolChanged {
        symbol: String,
    },
    BaseUriChanged {
        base_uri: String,
    },

    // Change Event for collection config
    SupplyLimitChanged {
        supply_limit: u128,
    },
    MintLimitChanged {
        mint_limit: u128,
    },
    MintPriceChanged {
        mint_price: u128,
    },
    SaleTimeChanged {
        sale_time: u64,
        sale_active: bool,
    },
    SaleActiveChanged {
        sale_active: bool,
    },
    WithdrawWalletChanged {
        withdraw_wallet: ActorId
    },

    TransferValue,
}

#[derive(Default, Debug, Encode, Decode, TypeInfo, Clone)]
pub struct TokenMetadata {
    // ex. "CryptoKitty #100"
    pub name: String,
    // free-form description
    pub description: String,
    // URL to associated media, preferably to decentralized, content-addressed storage
    pub media: String,
    // URL to an off-chain JSON file with more info.
    pub reference: String,
}

#[derive(Debug, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum DropspaceNftErr {
    Uninitialized,
    Unauthorized,
    MintLimitOver,
    SupplyLimitOver,
}

#[derive(Default, Debug, Encode, Decode, TypeInfo)]
pub struct State {
    pub owner_by_id: Vec<(TokenId, ActorId)>,
    pub token_approvals: Vec<(TokenId, ActorId)>,
    pub token_metadata_by_id: Vec<(TokenId, TokenMetadata)>,
    pub tokens_for_owner: Vec<(ActorId, Vec<TokenId>)>,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub collection: Collection,
    pub config: Config,
}

#[derive(Default, Debug, Encode, Decode, TypeInfo)]
pub struct ProgramInfo {
    pub collection: Collection,
    pub config: Config,
    pub token_id: TokenId,
    pub sale_active: bool,
    pub total_supply: u128,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum StateQuery {
    All,
    Config,
    Collection,
    Owner,
    CurrentTokenId,
    OwnerById { token_id: TokenId },
    TokenApprovals { token_id: TokenId },
    TokenMetadata { token_id: TokenId },
    OwnerTokens { owner: ActorId },
    SaleActive,
    ProgramInfo,
}

#[derive(Encode, Decode, TypeInfo)]
pub enum StateReply {
    All(State),
    Config(Config),
    Collection(Collection),
    Owner(ActorId),
    CurrentTokenId(TokenId),
    OwnerById(Option<ActorId>),
    TokenApprovals(Option<ActorId>),
    TokenMetadata(Option<TokenMetadata>),
    OwnerTokens(Option<Vec<TokenId>>),
    SaleActive(bool),
    ProgramInfo(ProgramInfo),
}
