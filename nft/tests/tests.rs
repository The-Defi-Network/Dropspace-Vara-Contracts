use gstd::{ActorId, Encode};
use gtest::System;
mod utils;
use nft_io::*;
use utils::*;

const USERS: &[u64] = &[3, 4, 5, 6, 7];
const ZERO_ID: u64 = 0;

#[test]
fn mint_success() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let res = mint(&nft, USERS[0], USERS[1].into());
    let token_metadata = TokenMetadata {
        name: "CryptoKitty".to_string(),
        description: "Description".to_string(),
        media: "http://".to_string(),
        reference: "http://".to_string(),
    };
    let message = NftEvent::Minted {
        to: USERS[1].into(),
        token_metadata,
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.owner_by_id, [(0_u128, USERS[1].into())]);
    assert_eq!(state.tokens_for_owner, [(USERS[1].into(), vec![0])]);
}

#[test]
fn mint_failures() {
    let sys = System::new();
    sys.init_logger();
    let nft = gtest::Program::current_opt(&sys);

    let collection = Collection {
        name: String::from("MyToken"),
        description: String::from("My token"),
        symbol: String::from("My token"),
        base_uri: String::from("https://mynft-test.com/"),
    };

    let init_nft = InitNft {
        collection,
        config: Config {
            supply_limit: 1,
            mint_price: 2,
            mint_fee: 1,
            dev_wallet: USERS[3].into(),
            withdraw_wallet: USERS[4].into(),
            mint_limit: 1,
            sale_time: 0,
        },
    };

    let res = nft.send(USERS[0], init_nft);
    assert!(!res.main_failed());

    // zero address
    let res = mint(&nft, USERS[0], 0.into());
    assert!(res.main_failed());

    // limit_exceed
    let nft = sys.get_program(1);
    let res = mint(&nft, USERS[0], USERS[1].into());
    assert!(!res.main_failed());
    let res = mint(&nft, USERS[0], USERS[1].into());
    assert!(res.main_failed())
}

#[test]
fn burn_success() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    let res = mint(&nft, USERS[0], USERS[1].into());
    assert!(!res.main_failed());
    let res = burn(&nft, USERS[1], 0);
    let message = NftEvent::Burnt { token_id: 0 }.encode();
    assert!(res.contains(&(USERS[1], message)));

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert!(state.owner_by_id.is_empty());
    assert!(state.tokens_for_owner.is_empty());
}

#[test]
fn burn_failures() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    mint(&nft, USERS[0], USERS[1].into());
    // must fail since the token doesn't exist
    assert!(burn(&nft, USERS[1], 1).main_failed());
    // must fail since the caller is not the token owner
    assert!(burn(&nft, USERS[0], 0).main_failed());
}

#[test]
fn transfer_success() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    assert!(!mint(&nft, USERS[0], USERS[1].into()).main_failed());

    let res = transfer(&nft, USERS[1], USERS[2], 0);
    let message = NftEvent::Transferred {
        from: USERS[1].into(),
        to: USERS[2].into(),
        token_id: 0,
    }
    .encode();
    assert!(res.contains(&(USERS[1], message)));

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.owner_by_id, [(0_u128, USERS[2].into())]);
    assert_eq!(state.tokens_for_owner, [(USERS[2].into(), vec![0])]);
}

#[test]
fn transfer_failures() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    assert!(!mint(&nft, USERS[0], USERS[1].into()).main_failed());

    // must fail since the token doesn't exist
    assert!(transfer(&nft, USERS[1], USERS[2], 1).main_failed());
    // must fail since the caller is not the token owner
    assert!(transfer(&nft, USERS[0], USERS[2], 0).main_failed());
    // must fail since transfer to the zero address
    assert!(transfer(&nft, USERS[1], ZERO_ID, 0).main_failed());
}

#[test]
fn approve_success() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    assert!(!mint(&nft, USERS[0], USERS[1].into()).main_failed());
    let res = approve(&nft, USERS[1], USERS[2], 0);
    let message = NftEvent::Approved {
        owner: USERS[1].into(),
        approved_account: USERS[2].into(),
        token_id: 0,
    }
    .encode();
    assert!(res.contains(&(USERS[1], message)));
    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.token_approvals, [(0_u128, USERS[2].into())]);

    assert!(!transfer(&nft, USERS[2], USERS[0], 0).main_failed());

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert!(state.token_approvals.is_empty());
}

#[test]
fn approve_failures() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    assert!(!mint(&nft, USERS[0], USERS[1].into()).main_failed());
    // must fail since the token doesn't exist
    assert!(approve(&nft, USERS[1], USERS[2], 1).main_failed());
    // must fail since the caller is not the token owner
    assert!(approve(&nft, USERS[0], USERS[2], 0).main_failed());
    // must fail since approval to the zero address
    assert!(approve(&nft, USERS[1], ZERO_ID, 0).main_failed());

    //approve
    assert!(!approve(&nft, USERS[1], USERS[2], 0).main_failed());
    //transfer
    assert!(!transfer(&nft, USERS[1], USERS[0], 0).main_failed());
    //must fail since approval was removed after transferring
    assert!(transfer(&nft, USERS[2], USERS[0], 0).main_failed());
}

#[test]
fn owner_success() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    assert!(!mint(&nft, USERS[0], USERS[1].into()).main_failed());
    let res = owner_of(&nft, USERS[1], 0);

    let message = NftEvent::Owner {
        token_id: 0,
        owner: ActorId::from(USERS[1]),
    }
    .encode();
    assert!(res.contains(&(USERS[1], message)));
}

#[test]
fn owner_failure() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    assert!(!mint(&nft, USERS[0], USERS[1].into()).main_failed());
    let res = owner_of(&nft, USERS[1], 1);
    assert!(res.main_failed());
}

#[test]
fn is_approved_to_success() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    assert!(!mint(&nft, USERS[0], USERS[1].into()).main_failed());
    assert!(!approve(&nft, USERS[1], USERS[2], 0).main_failed());

    let res = is_approved_to(&nft, USERS[0], 0, USERS[2]);
    let message = NftEvent::CheckIfApproved {
        to: USERS[2].into(),
        token_id: 0,
        approved: true,
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));
}

#[test]
fn is_approved_to_failure() {
    let sys = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);
    assert!(!mint(&nft, USERS[0], USERS[1].into()).main_failed());
    assert!(!approve(&nft, USERS[1], USERS[2], 0).main_failed());
    let res = is_approved_to(&nft, USERS[1], 1, USERS[2]);
    assert!(res.main_failed());
}

#[test]
fn test_set_name() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let new_name = "New Name".to_string();
    let res = nft.send(
        USERS[0],
        NftAction::SetName {
            name: new_name.clone(),
        },
    );
    let message = NftEvent::NameChanged {
        name: new_name.clone(),
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.collection.name, new_name);

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetName {
            name: new_name.clone(),
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_set_description() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let new_description = "New Description".to_string();
    let res = nft.send(
        USERS[0],
        NftAction::SetDescription {
            description: new_description.clone(),
        },
    );
    let message = NftEvent::DescriptionChanged {
        description: new_description.clone(),
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.collection.description, new_description);

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetDescription {
            description: new_description.clone(),
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_set_symbol() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let new_symbol = "New symbol".to_string();
    let res = nft.send(
        USERS[0],
        NftAction::SetSymbol {
            symbol: new_symbol.clone(),
        },
    );
    let message = NftEvent::SymbolChanged {
        symbol: new_symbol.clone(),
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.collection.symbol, new_symbol);

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetSymbol {
            symbol: new_symbol.clone(),
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_set_base_uri() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let new_base_uri = "https://new-base.com".to_string();
    let res = nft.send(
        USERS[0],
        NftAction::SetBaseUri {
            base_uri: new_base_uri.clone(),
        },
    );
    let message = NftEvent::BaseUriChanged {
        base_uri: new_base_uri.clone(),
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.collection.base_uri, new_base_uri);

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetBaseUri {
            base_uri: new_base_uri.clone(),
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_set_withdraw_wallet() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let new_withdraw_wallet = USERS[4].into();
    let res = nft.send(
        USERS[0],
        NftAction::SetWithdrawWallet {
            withdraw_wallet: new_withdraw_wallet,
        },
    );
    let message = NftEvent::WithdrawWalletChanged {
        withdraw_wallet: new_withdraw_wallet,
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.config.withdraw_wallet, new_withdraw_wallet.into());

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetWithdrawWallet {
            withdraw_wallet: new_withdraw_wallet,
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_set_supply_limit() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let new_supply_limit = 101;
    let res = nft.send(
        USERS[0],
        NftAction::SetSupplyLimit {
            supply_limit: new_supply_limit,
        },
    );
    let message = NftEvent::SupplyLimitChanged {
        supply_limit: new_supply_limit,
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    //todo:  Read test here
    //

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.config.supply_limit, new_supply_limit);

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetSupplyLimit {
            supply_limit: new_supply_limit,
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_set_mint_limit() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let new_mint_limit = 101;
    let res = nft.send(
        USERS[0],
        NftAction::SetMintLimit {
            mint_limit: new_mint_limit,
        },
    );
    let message = NftEvent::MintLimitChanged {
        mint_limit: new_mint_limit,
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    //todo:  Read test here
    //

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.config.mint_limit, new_mint_limit);

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetMintLimit {
            mint_limit: new_mint_limit,
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_set_mint_price() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    let new_mint_price = 101;
    let res = nft.send(
        USERS[0],
        NftAction::SetMintPrice {
            mint_price: new_mint_price,
        },
    );
    let message = NftEvent::MintPriceChanged {
        mint_price: new_mint_price,
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    //todo:  Read test here
    //

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.config.mint_price, new_mint_price);

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetMintPrice {
            mint_price: new_mint_price,
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_set_sale_time() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft: gtest::Program = sys.get_program(1);

    let new_sale_time = 1719561334;
    let res: gtest::RunResult = nft.send(
        USERS[0],
        NftAction::SetSaleTime {
            sale_time: new_sale_time,
        },
    );
    let message = NftEvent::SaleTimeChanged {
        sale_time: new_sale_time,
        sale_active: new_sale_time <= sys.block_timestamp() / 1000,
    }
    .encode();
    assert!(res.contains(&(USERS[0], message)));

    // println!("\n-------------{}\n------------\n", sys.block_timestamp());
    // dbg!(sys.block_timestamp());
    //todo:  Read test here
    //

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.config.sale_time, new_sale_time);

    // Not authorized test
    let res = nft.send(
        USERS[1],
        NftAction::SetSaleTime {
            sale_time: new_sale_time,
        },
    );
    assert!(res.main_failed());
}

#[test]
fn test_toggle_sale_active() {
    let sys: System = System::new();
    init_nft(&sys);
    let nft = sys.get_program(1);

    // Set saleActive = true
    let _res = nft.send(USERS[0], NftAction::SetSaleTime { sale_time: 0 });

    let state = get_state(&nft).expect("Unexpected invalid state.");
    assert_eq!(state.config.sale_time, 0);

    // Toggle sale active 1
    let _res = nft.send(USERS[0], NftAction::ToggleSaleActive {});

    let program_info: ProgramInfo = get_program_info(&nft).expect("Unexpected invalid state.");
    assert_eq!(program_info.sale_active, false);
    
    // Toggle sale active 2
    let _res = nft.send(USERS[0], NftAction::ToggleSaleActive {});
    dbg!(_res);

    let program_info: ProgramInfo = get_program_info(&nft).expect("Unexpected invalid state.");
    assert_eq!(program_info.sale_active, true);
    

    // Not authorized test
    let _res = nft.send(
        USERS[1],
        NftAction::SetSaleTime {
            sale_time: 0,
        },
    );

    assert!(_res.main_failed());
}

#[test]
fn test_buy() {
    let sys = System::new();
    init_nft(&sys);
    let mut nft = sys.get_program(1);
    
    // nft.mint(100_000_000_000_000);
    sys.mint_to(USERS[0], 200_000_000_000_000);
    sys.mint_to(USERS[3], 100_000_000_000_000); // faucet to a dev wallet
    sys.mint_to(USERS[4], 100_000_000_000_000); // faucet to a withdraw wallet
    
    // Buy.
    let qty = 10;
    let res = buy(&nft, USERS[0], qty);
    
    // dbg!(res.clone());
    // dbg!(sys.balance_of(USERS[0]));

    let message: Vec<u8> = NftEvent::Bought { 
        to: USERS[0].into(),
        amount: qty,
    }
    .encode();

    assert!(res.contains(&(USERS[0], message)));


    // Checking balances
    let info = get_program_info(&nft).unwrap();

    // check buyer's balance
    let mut bal = sys.balance_of(USERS[0]);
    assert_eq!(bal, 200_000_000_000_000 - info.config.mint_price.saturating_mul(qty) - info.config.mint_fee.saturating_mul(qty));

    
    bal = nft.balance();
    dbg!(bal);

    // check dev wallet's balance
    bal = sys.balance_of(USERS[3]);
    assert_eq!(bal, 100_000_000_000_000 + info.config.mint_fee.saturating_mul(qty));

    // check withdrawal wallet's balance
    bal = sys.balance_of(USERS[4]);
    assert_eq!(bal, 100_000_000_000_000 + info.config.mint_price.saturating_mul(qty));

}


#[test]
fn test_read_token_metadata() {
    let sys = System::new();
    init_nft_airdrop(&sys);
    let nft = sys.get_program(1);
    
    // Buy 2 NFTs.
    let res = buy(&nft, USERS[0], 2);
    
    let message: Vec<u8> = NftEvent::Bought { 
        to: USERS[0].into(),
        amount: 2,
    }
    .encode();

    assert!(res.contains(&(USERS[0], message)));


    let _res = get_token_meta(&nft, 0).expect("Unexpected invalid state.");
    assert!(_res.reference == String::from("https://mynft-test.com/0"));    
    
    let _res = get_token_meta(&nft, 1).expect("Unexpected invalid state.");
    assert!(_res.reference == String::from("https://mynft-test.com/1"));    

}