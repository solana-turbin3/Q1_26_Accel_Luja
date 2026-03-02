use std::path::PathBuf;

use litesvm::LiteSVM;
use litesvm_token::{
    CreateAssociatedTokenAccount, CreateMint, MintTo,
    spl_token::{self},
};

use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;
use spl_associated_token_account::get_associated_token_address;

pub const PROGRAM_ID: &str = "9MBbGAcz2KvydYUGVCRNr8ZhiWPpSFXNhE4p8mw61egr";
pub const TOKEN_PROGRAM_ID: Pubkey = spl_token::ID;
pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

pub fn program_id() -> Pubkey {
    crate::ID
}

pub fn setup() -> (LiteSVM, Keypair) {
    let mut svm = LiteSVM::new();
    let payer = Keypair::new();

    svm.airdrop(&payer.pubkey(), 10 * LAMPORTS_PER_SOL)
        .expect("Airdrop failed");

    // Load program SO file
    println!("The path is!! {}", env!("CARGO_MANIFEST_DIR"));
    let so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/sbpf-solana-solana/release/pinocchio_fundraiser.so");
    println!("Full path: {}", so_path.display());
    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

    svm.add_program(program_id(), &program_data)
        .expect("Failed to add program");

    (svm, payer)
}

pub struct InitializeResponse {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub mint: Pubkey,
    pub fundraiser_account: Pubkey,
    pub fundraiser_ata: Pubkey,
    pub maker_ata: Pubkey,
    pub initialize_cu: u64,
}

pub struct ContributeResponse {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub contributor: Keypair,
    pub mint: Pubkey,
    pub fundraiser_account: Pubkey,
    pub contributor_account: Pubkey,
    pub fundraiser_ata: Pubkey,
    pub contributor_ata: Pubkey,
    pub maker_ata: Pubkey,
    pub contribute_cu: u64,
}

pub fn initialize_ix() -> InitializeResponse {
    let (mut svm, payer) = setup();
    assert_eq!(program_id().to_string(), PROGRAM_ID);

    let program_id = program_id();

    let mint = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let maker_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint)
        .owner(&payer.pubkey())
        .send()
        .unwrap();

    let fundraiser_account =
        Pubkey::find_program_address(&[b"fundraiser", payer.pubkey().as_ref()], &program_id);

    let fundraiser_ata = get_associated_token_address(&fundraiser_account.0, &mint);

    let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
    let token_program = TOKEN_PROGRAM_ID;
    let system_program = solana_sdk_ids::system_program::ID;

    let amount_to_raise: u64 = 10_000_000;
    let duration: u8 = 0;
    let bump: u8 = fundraiser_account.1;

    let initialize_data = [
        vec![0u8],
        amount_to_raise.to_le_bytes().to_vec(),
        duration.to_le_bytes().to_vec(),
        bump.to_le_bytes().to_vec(),
    ]
    .concat();

    let initialize_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(mint, false),
            AccountMeta::new(fundraiser_account.0, false),
            AccountMeta::new(fundraiser_ata, false),
            AccountMeta::new(token_program, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(associated_token_program, false),
        ],
        data: initialize_data,
    };
    let message = Message::new(&[initialize_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let tx = svm.send_transaction(transaction).unwrap();

    // println!("TX:{:?}", tx);

    InitializeResponse {
        svm,
        maker: payer,
        mint,
        fundraiser_account: fundraiser_account.0,
        fundraiser_ata,
        maker_ata,
        initialize_cu: tx.compute_units_consumed,
    }
}

pub fn contribute_ix() -> ContributeResponse {
    let mut s = initialize_ix();
    let program_id = program_id();
    let contributor = Keypair::new();

    let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
    let token_program = TOKEN_PROGRAM_ID;
    let system_program = solana_sdk_ids::system_program::ID;

    s.svm
        .airdrop(&contributor.pubkey(), 10 * LAMPORTS_PER_SOL)
        .unwrap();

    let contributor_account = Pubkey::find_program_address(
        &[
            b"contributor",
            s.fundraiser_account.as_ref(),
            contributor.pubkey().as_ref(),
        ],
        &program_id,
    );

    let contributor_ata = CreateAssociatedTokenAccount::new(&mut s.svm, &contributor, &s.mint)
        .owner(&contributor.pubkey())
        .send()
        .unwrap();

    MintTo::new(
        &mut s.svm,
        &s.maker,
        &s.mint,
        &contributor_ata,
        100_000_000_000,
    )
    .send()
    .unwrap();

    let amount_to_contribute: u64 = 10_000_000;
    let bump: u8 = contributor_account.1;
    let contribute_data = [
        vec![1u8],
        amount_to_contribute.to_le_bytes().to_vec(),
        bump.to_le_bytes().to_vec(),
    ]
    .concat();

    let contribute_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(contributor.pubkey(), true),
            AccountMeta::new(s.mint, false),
            AccountMeta::new(s.fundraiser_account, false),
            AccountMeta::new(contributor_account.0, false),
            AccountMeta::new(contributor_ata, false),
            AccountMeta::new(s.fundraiser_ata, false),
            AccountMeta::new(token_program, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(associated_token_program, false),
        ],
        data: contribute_data,
    };

    let message = Message::new(&[contribute_ix], Some(&contributor.pubkey()));
    let recent_blockhash = s.svm.latest_blockhash();
    let transaction = Transaction::new(&[&contributor], message, recent_blockhash);

    let tx = s.svm.send_transaction(transaction).unwrap();

    ContributeResponse {
        svm: s.svm,
        maker: s.maker,
        contributor,
        mint: s.mint,
        fundraiser_account: s.fundraiser_account,
        contributor_account: contributor_account.0,
        fundraiser_ata: s.fundraiser_ata,
        contributor_ata,
        maker_ata: s.maker_ata,
        contribute_cu: tx.compute_units_consumed,
    }
}
