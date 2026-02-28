use std::path::PathBuf;

use litesvm::LiteSVM;
use litesvm_token::{
    spl_token::{self},
    CreateAssociatedTokenAccount, CreateMint, MintTo,
};

use solana_instruction::{AccountMeta, Instruction};
use solana_keypair::Keypair;
use solana_message::Message;
use solana_native_token::LAMPORTS_PER_SOL;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

pub const PROGRAM_ID: &str = "99AW8S9fD1QREzbE25W3uwo7DyjQvuzsYDfcdD6GZbVv";
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
        .join("target/sbpf-solana-solana/release/pinocchio_escrow.so");
    println!("Full path: {}", so_path.display());
    let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

    svm.add_program(program_id(), &program_data)
        .expect("Failed to add program");

    (svm, payer)
}

pub struct Make {
    pub svm: LiteSVM,
    pub maker: Keypair,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub escrow_pda: Pubkey,
    pub escrow_ata: Pubkey,
    pub maker_ata_a: Pubkey,
}

pub fn make_ix() -> Make {
    let (mut svm, payer) = setup();
    let program_id = program_id();

    assert_eq!(program_id.to_string(), PROGRAM_ID);
    let mint_a = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();
    let mint_b = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint_a)
        .owner(&payer.pubkey())
        .send()
        .unwrap();

    let escrow = Pubkey::find_program_address(&[b"escrow", payer.pubkey().as_ref()], &program_id);

    let escrow_ata = spl_associated_token_account::get_associated_token_address(&escrow.0, &mint_a);

    let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
    let token_program = TOKEN_PROGRAM_ID;
    let system_program = solana_sdk_ids::system_program::ID;

    MintTo::new(&mut svm, &payer, &mint_a, &maker_ata_a, 1000000000)
        .send()
        .unwrap();

    let amount_to_receive: u64 = 100000000; // 100 tokens with 6 decimal places
    let amount_to_give: u64 = 500000000; // 500 tokens with 6 decimal places
    let bump: u8 = escrow.1;

    let make_data = [
        vec![0u8],
        amount_to_receive.to_le_bytes().to_vec(),
        amount_to_give.to_le_bytes().to_vec(),
        bump.to_le_bytes().to_vec(),
    ]
    .concat();

    let make_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(mint_a, false),
            AccountMeta::new(mint_b, false),
            AccountMeta::new(escrow.0, false),
            AccountMeta::new(maker_ata_a, false),
            AccountMeta::new(escrow_ata, false),
            AccountMeta::new(system_program, false),
            AccountMeta::new(token_program, false),
            AccountMeta::new(associated_token_program, false),
        ],
        data: make_data,
    };

    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = svm.latest_blockhash();
    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    let tx = svm.send_transaction(transaction).unwrap();
    println!("Tx completed:{:?}", tx);

    Make {
        svm,
        maker: payer,
        mint_a,
        mint_b,
        escrow_pda: escrow.0,
        escrow_ata,
        maker_ata_a,
    }
}
