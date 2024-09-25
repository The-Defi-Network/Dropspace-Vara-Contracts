#![no_std]

use gstd::{
    collections::{HashMap, HashSet},
    debug, exec, msg,
    prelude::*,
    ActorId,
};

use nft_io::*;

#[derive(Debug, Default)]
pub struct Nft {
    pub owner_by_id: HashMap<TokenId, ActorId>,
    pub token_approvals: HashMap<TokenId, ActorId>,
    pub token_metadata_by_id: HashMap<TokenId, TokenMetadata>,
    pub tokens_for_owner: HashMap<ActorId, HashSet<TokenId>>,
    pub token_id: TokenId,
    pub owner: ActorId,
    pub collection: Collection,
    pub config: Config,
}

const SALE_TIME_MAX: u64 = u64::MAX;

static mut NFT: Option<Nft> = None;

#[no_mangle]
unsafe extern "C" fn init() {
    let init: InitNft = msg::load().expect("Unable to decode InitNft");

    let nft = Nft {
        collection: init.collection,
        config: init.config,
        owner: msg::source(),
        ..Default::default()
    };
    NFT = Some(nft);
}

impl Nft {
    fn buy(&mut self, amount: u128) -> NftEvent {
        let source: ActorId = msg::source();
        let value = msg::value();

        debug!("Funds: {}", value);
        debug!("Buying Qty: {}", amount);

        self.check_config();
        self.check_zero_address(&source);

        let sale_active = self.sale_active();
        if !sale_active {
            panic!("Sale is not active!");
        }

        let total_supply: u128 = self.token_metadata_by_id.len() as u128;

        if amount <= 0 {
            panic!("Amount {} invalid!", amount);
        }

        if amount > self.config.mint_limit {
            panic!("Mint limit {} Over!", self.config.mint_limit);
        }

        if total_supply.saturating_add(amount) > self.config.supply_limit {
            panic!("Supply limit {} Over!", self.config.supply_limit);
        }

        let required_value = self
            .config
            .mint_price
            .saturating_mul(amount)
            .saturating_add(self.config.mint_fee.saturating_mul(amount));

        if required_value > value {
            panic!("Funds insufficient!");
        }

        if self.config.mint_price > 0 {
            debug!(
                "Sending to withdraw wallet: {}",
                self.config.mint_price.saturating_mul(amount)
            );
            msg::send(
                self.config.withdraw_wallet,
                NftEvent::TransferValue,
                self.config.mint_price.saturating_mul(amount),
            )
            .expect("Failed to send funds to withdrawal wallet!");

            debug!("Sent");
        }

        if self.config.mint_fee > 0 {
            debug!(
                "Sending to dev wallet: {}",
                self.config.mint_fee.saturating_mul(amount)
            );
            
            msg::send(
                self.config.dev_wallet,
                NftEvent::TransferValue,
                self.config.mint_fee.saturating_mul(amount),
            )
            .expect("Failed to send funds to dev wallet!");

            debug!("Sent");
        }

        let remainder = value.saturating_sub(required_value);
        if remainder > 0 {
            debug!("Sending back : {}", remainder);
            msg::send(source, NftEvent::TransferValue, remainder)
                .expect("Failed to send funds back to user!");
        }

        for _i in 0..amount {
            self.mint_single(&source);
        }

        NftEvent::Bought { to: source, amount }
    }

    fn reserve(&mut self, amount: u128) -> NftEvent {
        self.check_collection_owner();

        let source: ActorId = msg::source();
        let value = msg::value();
        
        debug!("Funds: {}", value);
        debug!("Reserving Qty: {}", amount);

        self.check_config();
        self.check_zero_address(&source);

        let sale_active = self.sale_active();
        if !sale_active {
            panic!("Sale is not active!");
        }

        let total_supply: u128 = self.token_metadata_by_id.len() as u128;

        if amount <= 0 {
            panic!("Amount {} invalid!", amount);
        }

        if amount > self.config.mint_limit {
            panic!("Mint limit {} Over!", self.config.mint_limit);
        }

        if total_supply.saturating_add(amount) > self.config.supply_limit {
            panic!("Supply limit {} Over!", self.config.supply_limit);
        }

        for _i in 0..amount {
            self.mint_single(&source);
        }

        NftEvent::Reserved { to: source, amount }
    }

    /// Mint a new nft using `TokenMetadata`
    fn mint_single(&mut self, to: &ActorId) -> NftEvent {
        self.owner_by_id.insert(self.token_id, *to);

        self.tokens_for_owner
            .entry(*to)
            .and_modify(|tokens| {
                tokens.insert(self.token_id);
            })
            .or_insert_with(|| HashSet::from([self.token_id]));

        let token_metadata = TokenMetadata {
            name: self.token_id.to_string(),
            description: "".to_string(),
            media: "".to_string(),
            reference: "".to_string(),
        };

        self.token_metadata_by_id.insert(
            self.token_id,
            token_metadata.clone(),
        );

        self.token_id += 1;

        NftEvent::Minted {
            to: *to,
            token_metadata,
        }
    }

    /// Mint a new nft using `TokenMetadata`
    /* fn mint(&mut self, to: &ActorId, token_metadata: TokenMetadata) -> NftEvent {
        self.check_config();
        self.check_zero_address(to);
        self.owner_by_id.insert(self.token_id, *to);
        self.tokens_for_owner
            .entry(*to)
            .and_modify(|tokens| {
                tokens.insert(self.token_id);
            })
            .or_insert_with(|| HashSet::from([self.token_id]));
        self.token_metadata_by_id
            .insert(self.token_id, token_metadata.clone());

        self.token_id += 1;

        NftEvent::Minted {
            to: *to,
            token_metadata,
        }
    } */

    /// Burn nft by `TokenId`
    fn burn(&mut self, token_id: TokenId) -> NftEvent {
        let owner = *self
            .owner_by_id
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");

        self.check_owner(&owner);
        self.owner_by_id.remove(&token_id);
        self.token_metadata_by_id.remove(&token_id);

        if let Some(tokens) = self.tokens_for_owner.get_mut(&owner) {
            tokens.remove(&token_id);
            if tokens.is_empty() {
                self.tokens_for_owner.remove(&owner);
            }
        }
        self.token_approvals.remove(&token_id);

        NftEvent::Burnt { token_id }
    }
    ///  Transfer token from `token_id` to address `to`
    fn transfer(&mut self, to: &ActorId, token_id: TokenId) -> NftEvent {
        let owner = *self
            .owner_by_id
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");

        self.can_transfer(token_id, &owner);
        self.check_zero_address(to);
        // assign new owner
        self.owner_by_id
            .entry(token_id)
            .and_modify(|owner| *owner = *to);
        // push token to new owner
        self.tokens_for_owner
            .entry(*to)
            .and_modify(|tokens| {
                tokens.insert(token_id);
            })
            .or_insert_with(|| HashSet::from([token_id]));
        // remove token from old owner
        if let Some(tokens) = self.tokens_for_owner.get_mut(&owner) {
            tokens.remove(&token_id);
            if tokens.is_empty() {
                self.tokens_for_owner.remove(&owner);
            }
        }
        // remove approvals if any
        self.token_approvals.remove(&token_id);

        NftEvent::Transferred {
            from: owner,
            to: *to,
            token_id,
        }
    }
    ///  Approve token from `token_id` to address `to`
    fn approve(&mut self, to: &ActorId, token_id: TokenId) -> NftEvent {
        let owner = self
            .owner_by_id
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");
        self.check_owner(owner);
        self.check_zero_address(to);
        self.check_approve(&token_id);
        self.token_approvals.insert(token_id, *to);

        NftEvent::Approved {
            owner: *owner,
            approved_account: *to,
            token_id,
        }
    }
    /// Get `ActorId` of the nft owner with `token_id`
    fn owner(&self, token_id: TokenId) -> NftEvent {
        let owner = self
            .owner_by_id
            .get(&token_id)
            .expect("NonFungibleToken: token does not exist");

        NftEvent::Owner {
            owner: *owner,
            token_id,
        }
    }
    /// Get confirmation about approval to address `to` and `token_id`
    fn is_approved_to(&self, to: &ActorId, token_id: TokenId) -> NftEvent {
        if !self.owner_by_id.contains_key(&token_id) {
            panic!("Token does not exist")
        }
        self.token_approvals.get(&token_id).map_or_else(
            || NftEvent::CheckIfApproved {
                to: *to,
                token_id,
                approved: false,
            },
            |approval_id| NftEvent::CheckIfApproved {
                to: *to,
                token_id,
                approved: *approval_id == *to,
            },
        )
    }

    /// Checking the configuration with current contract data
    fn check_config(&self) {
        if self.config.supply_limit <= self.token_metadata_by_id.len() as u128 {
            panic!(
                "Mint impossible because max minting count {} limit exceeded",
                self.config.supply_limit
            );
        }
    }

    /// Check for ZERO_ID address
    fn check_zero_address(&self, account: &ActorId) {
        if account == &ZERO_ID {
            panic!("NonFungibleToken: zero address");
        }
    }

    /// Checks that `msg::source()` is the owner of the token with indicated `token_id`
    fn check_owner(&self, owner: &ActorId) {
        if owner != &msg::source() {
            panic!("NonFungibleToken: access denied");
        }
    }

    /// Checks that `msg::source()` is the owner of the collection
    fn check_collection_owner(&self) {
        if self.owner != msg::source() {
            panic!("NonFungibleToken: not authorized");
        }
    }

    /// Checks that `msg::source()` is allowed to manage the token with indicated `token_id`
    fn can_transfer(&self, token_id: TokenId, owner: &ActorId) {
        if let Some(approved_accounts) = self.token_approvals.get(&token_id) {
            if approved_accounts == &msg::source() {
                return;
            }
        }
        self.check_owner(owner);
    }
    /// Check the existence of a approve
    fn check_approve(&self, token_id: &TokenId) {
        if self.token_approvals.contains_key(token_id) {
            panic!("Approve has already been issued");
        }
    }

    /// Set collection's name
    fn set_name(&mut self, name: &String) -> NftEvent {
        self.check_collection_owner();
        self.collection.name = name.to_string();

        NftEvent::NameChanged {
            name: name.to_string(),
        }
    }

    /// Set collection's description
    fn set_description(&mut self, description: &String) -> NftEvent {
        self.check_collection_owner();
        self.collection.description = description.to_string();

        NftEvent::DescriptionChanged {
            description: description.to_string(),
        }
    }

    /// Set collection's symbol
    fn set_symbol(&mut self, symbol: &String) -> NftEvent {
        self.check_collection_owner();
        self.collection.symbol = symbol.to_string();

        NftEvent::SymbolChanged {
            symbol: symbol.to_string(),
        }
    }

    /// Set collection's base_uri
    fn set_base_uri(&mut self, base_uri: &String) -> NftEvent {
        self.check_collection_owner();
        self.collection.base_uri = base_uri.to_string();

        NftEvent::BaseUriChanged {
            base_uri: base_uri.to_string(),
        }
    }

    // Set withdraw_address.
    fn set_withdraw_wallet(&mut self, withdraw_wallet: &ActorId) -> NftEvent {
        self.check_collection_owner();
        self.config.withdraw_wallet = *withdraw_wallet;

        NftEvent::WithdrawWalletChanged {
            withdraw_wallet: *withdraw_wallet,
        }
    }

    /// Set supply limit.
    fn set_supply_limit(&mut self, supply_limit: u128) -> NftEvent {
        self.check_collection_owner();
        self.config.supply_limit = supply_limit;

        NftEvent::SupplyLimitChanged { supply_limit }
    }

    /// Set mint limit.
    fn set_mint_limit(&mut self, mint_limit: u128) -> NftEvent {
        self.check_collection_owner();
        self.config.mint_limit = mint_limit;

        NftEvent::MintLimitChanged { mint_limit }
    }

    /// Set mint price.
    fn set_mint_price(&mut self, mint_price: u128) -> NftEvent {
        self.check_collection_owner();
        self.config.mint_price = mint_price;

        NftEvent::MintPriceChanged { mint_price }
    }

    // Set sale time.
    fn set_sale_time(&mut self, sale_time: u64) -> NftEvent {
        self.check_collection_owner();
        self.config.sale_time = sale_time;

        NftEvent::SaleTimeChanged {
            sale_time,
            sale_active: sale_time <= exec::block_timestamp() / 1000,
        }
    }

    fn sale_active(&self) -> bool {
        let current_ts = exec::block_timestamp() / 1000;

        self.config.sale_time <= current_ts
    }

    fn toggle_sale_active(&mut self) -> NftEvent {
        self.check_collection_owner();

        let sale_active_status = self.sale_active();
        if sale_active_status {
            self.set_sale_time(SALE_TIME_MAX);
        } else {
            self.set_sale_time(0);
        }

        NftEvent::SaleActiveChanged {
            sale_active: !sale_active_status,
        }
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: NftAction = msg::load().expect("Could not load NftAction");
    let nft = unsafe { NFT.as_mut().expect("`NFT` is not initialized.") };
    let result = match action {
        // NftAction::Mint { to, token_metadata } => nft.mint(&to, token_metadata),
        NftAction::Burn { token_id } => nft.burn(token_id),
        NftAction::Transfer { to, token_id } => nft.transfer(&to, token_id),
        NftAction::Approve { to, token_id } => nft.approve(&to, token_id),
        NftAction::GetOwner { token_id } => nft.owner(token_id),
        NftAction::CheckIfApproved { to, token_id } => nft.is_approved_to(&to, token_id),
        // change collection info
        NftAction::SetName { name } => nft.set_name(&name),
        NftAction::SetDescription { description } => nft.set_description(&description),
        NftAction::SetSymbol { symbol } => nft.set_symbol(&symbol),
        NftAction::SetBaseUri { base_uri } => nft.set_base_uri(&base_uri),
        // change collection config
        NftAction::SetWithdrawWallet { withdraw_wallet } => {
            nft.set_withdraw_wallet(&withdraw_wallet)
        }
        NftAction::SetSupplyLimit { supply_limit } => nft.set_supply_limit(supply_limit),
        NftAction::SetMintLimit { mint_limit } => nft.set_mint_limit(mint_limit),
        NftAction::SetMintPrice { mint_price } => nft.set_mint_price(mint_price),
        NftAction::SetSaleTime { sale_time } => nft.set_sale_time(sale_time),
        NftAction::ToggleSaleActive {} => nft.toggle_sale_active(),
        NftAction::Buy { amount } => nft.buy(amount),
        NftAction::Reserve { amount } => nft.reserve(amount),
    };
    msg::reply(result, 0).expect("Failed to encode or reply with `NftEvent`.");
}

#[no_mangle]
extern "C" fn state() {
    let nft = unsafe { NFT.take().expect("Unexpected error in taking state") };
    let query: StateQuery = msg::load().expect("Unable to load the state query");
    match query {
        StateQuery::All => {
            msg::reply(StateReply::All(nft.into()), 0).expect("Unable to share the state");
        }
        StateQuery::Config => {
            msg::reply(StateReply::Config(nft.config), 0).expect("Unable to share the state");
        }
        StateQuery::Collection => {
            msg::reply(StateReply::Collection(nft.collection), 0)
                .expect("Unable to share the state");
        }
        StateQuery::Owner => {
            msg::reply(StateReply::Owner(nft.owner), 0).expect("Unable to share the state");
        }
        StateQuery::CurrentTokenId => {
            msg::reply(StateReply::CurrentTokenId(nft.token_id), 0)
                .expect("Unable to share the state");
        }
        StateQuery::OwnerById { token_id } => {
            msg::reply(
                StateReply::OwnerById(nft.owner_by_id.get(&token_id).cloned()),
                0,
            )
            .expect("Unable to share the state");
        }
        StateQuery::TokenApprovals { token_id } => {
            let approval = nft.token_approvals.get(&token_id).cloned();
            msg::reply(StateReply::TokenApprovals(approval), 0).expect("Unable to share the state");
        }
        StateQuery::TokenMetadata { token_id } => {
            let token_metadata = nft.token_metadata_by_id.get(&token_id).cloned().unwrap();
            msg::reply(
                // StateReply::TokenMetadata(nft.token_metadata_by_id.get(&token_id).cloned()),
                StateReply::TokenMetadata(Some(TokenMetadata {
                    name: token_metadata.name,
                    description: "".to_string(),
                    media: "".to_string(),
                    reference: nft.collection.base_uri.to_string() + &token_id.to_string(),
                })),
                0,
            )
            .expect("Unable to share the state");
        }
        StateQuery::OwnerTokens { owner } => {
            let tokens = nft
                .tokens_for_owner
                .get(&owner)
                .map(|hashset| hashset.iter().cloned().collect());
            msg::reply(StateReply::OwnerTokens(tokens), 0).expect("Unable to share the state");
        }
        StateQuery::SaleActive => {
            msg::reply(StateReply::SaleActive(nft.sale_active()), 0)
                .expect("Unable to share the state");
        }
        StateQuery::ProgramInfo => {
            let sale_active_status = nft.sale_active();

            msg::reply(
                StateReply::ProgramInfo(ProgramInfo {
                    collection: nft.collection,
                    config: nft.config,
                    token_id: nft.token_id,
                    sale_active: sale_active_status,
                    total_supply: nft.token_metadata_by_id.len() as u128,
                }),
                0,
            )
            .expect("Unable to share the state");
        }
    }
}

impl From<Nft> for State {
    fn from(value: Nft) -> Self {
        let Nft {
            owner_by_id,
            token_approvals,
            token_metadata_by_id,
            tokens_for_owner,
            token_id,
            owner,
            collection,
            config,
        } = value;

        let owner_by_id = owner_by_id.into_iter().collect();

        let token_approvals = token_approvals.into_iter().collect();

        let token_metadata_by_id = token_metadata_by_id.into_iter().collect();

        let tokens_for_owner = tokens_for_owner
            .into_iter()
            .map(|(id, tokens)| (id, tokens.into_iter().collect()))
            .collect();

        Self {
            owner_by_id,
            token_approvals,
            token_metadata_by_id,
            tokens_for_owner,
            token_id,
            owner,
            collection,
            config,
        }
    }
}
