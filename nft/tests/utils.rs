use gstd::ActorId;
use gtest::{Program, RunResult, System};
use nft_io::*;

const USERS: &[u64] = &[3, 4, 5, 6, 7];

pub fn init_nft(sys: &System) {
    sys.init_logger();
    let nft: Program = Program::current_opt(sys);

    let collection = Collection {
        name: String::from("MyToken"),
        description: String::from("My token"),
        symbol: String::from("My Symbol"),
        base_uri: String::from("https://mynft-test.com/"),
    };

    let init_nft = InitNft {
        collection,
        config: Config {
            supply_limit: 100,
            mint_price: 2_000_000_000_000,
            mint_fee: 1_000_000_000_000,
            dev_wallet: USERS[3].into(),
            withdraw_wallet: USERS[4].into(),
            mint_limit: 50,
            sale_time: 0,
        },
    };

    let res = nft.send(USERS[0], init_nft);

    assert!(!res.main_failed());
}

pub fn init_nft_airdrop(sys: &System) {
    sys.init_logger();
    let nft: Program = Program::current_opt(sys);

    let collection = Collection {
        name: String::from("MyToken"),
        description: String::from("My token"),
        symbol: String::from("My Symbol"),
        base_uri: String::from("https://mynft-test.com/"),
    };

    let init_nft = InitNft {
        collection,
        config: Config {
            supply_limit: 100,
            mint_price: 0,
            mint_fee: 0,
            dev_wallet: USERS[3].into(),
            withdraw_wallet: USERS[4].into(),
            mint_limit: 5,
            sale_time: 0,
        },
    };

    let res = nft.send(USERS[0], init_nft);

    assert!(!res.main_failed());
}

pub fn buy(nft: &Program<'_>, member: u64, amount: u128) -> RunResult {
    let info = get_program_info(nft).unwrap();

    // Mint price total
    let mut required_funds = info.config.mint_price.saturating_mul(amount);

    // Add mint fee total
    required_funds = required_funds.saturating_add(info.config.mint_fee.saturating_mul(amount));

    dbg!(required_funds);

    nft.send_with_value(member, NftAction::Buy { amount }, required_funds)
}

pub fn mint(nft: &Program<'_>, member: u64, to: ActorId) -> RunResult {
    nft.send(
        member,
        NftAction::Mint {
            to,
            token_metadata: TokenMetadata {
                name: "CryptoKitty".to_string(),
                description: "Description".to_string(),
                media: "http://".to_string(),
                reference: "http://".to_string(),
            },
        },
    )
}

pub fn burn(nft: &Program<'_>, member: u64, token_id: u64) -> RunResult {
    nft.send(
        member,
        NftAction::Burn {
            token_id: token_id.into(),
        },
    )
}

pub fn transfer(nft: &Program<'_>, from: u64, to: u64, token_id: u64) -> RunResult {
    nft.send(
        from,
        NftAction::Transfer {
            to: to.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn owner_of(nft: &Program<'_>, from: u64, token_id: u64) -> RunResult {
    nft.send(
        from,
        NftAction::GetOwner {
            token_id: token_id.into(),
        },
    )
}

pub fn is_approved_to(nft: &Program<'_>, from: u64, token_id: u64, to: u64) -> RunResult {
    nft.send(
        from,
        NftAction::CheckIfApproved {
            to: to.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn approve(nft: &Program<'_>, from: u64, to: u64, token_id: u64) -> RunResult {
    nft.send(
        from,
        NftAction::Approve {
            to: to.into(),
            token_id: token_id.into(),
        },
    )
}

pub fn get_state(nft: &Program<'_>) -> Option<State> {
    let reply = nft
        .read_state(StateQuery::All)
        .expect("Unexpected invalid reply.");
    if let StateReply::All(state) = reply {
        Some(state)
    } else {
        None
    }
}

pub fn get_program_info(nft: &Program<'_>) -> Option<ProgramInfo> {
    let reply = nft
        .read_state(StateQuery::ProgramInfo)
        .expect("Unexpected invalid reply.");

    if let StateReply::ProgramInfo(state) = reply {
        Some(state)
    } else {
        None
    }
}

pub fn get_token_meta(nft: &Program<'_>, token_id: u128) -> Option<TokenMetadata> {
    let reply = nft
        .read_state(StateQuery::TokenMetadata { token_id })
        .expect("Unexpected invalid reply.");

    if let StateReply::TokenMetadata(state) = reply {
        state
    } else {
        None
    }
}
