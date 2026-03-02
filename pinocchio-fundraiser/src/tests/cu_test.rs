use solana_message::{AccountMeta, Instruction, Message};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::tests::helper::{
    ASSOCIATED_TOKEN_PROGRAM_ID, ContributeResponse, TOKEN_PROGRAM_ID, contribute_ix,
    initialize_ix, program_id,
};

fn do_checker(s: &mut ContributeResponse) -> u64 {
    let program_id = program_id();
    let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
    let token_program = TOKEN_PROGRAM_ID;
    let system_program = solana_sdk_ids::system_program::ID;

    let checker_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(s.maker.pubkey(), true),
            AccountMeta::new(s.mint, false),
            AccountMeta::new(s.fundraiser_account, false),
            AccountMeta::new(s.fundraiser_ata, false),
            AccountMeta::new(s.maker_ata, false),
            AccountMeta::new(token_program, false),
            AccountMeta::new(associated_token_program, false),
            AccountMeta::new(system_program, false),
        ],
        data: vec![2u8],
    };

    let message = Message::new(&[checker_ix], Some(&s.maker.pubkey()));
    let recent_blockhash = s.svm.latest_blockhash();
    let transaction = Transaction::new(&[&s.maker], message, recent_blockhash);

    s.svm
        .send_transaction(transaction)
        .unwrap()
        .compute_units_consumed
}

fn do_refund(s: &mut ContributeResponse) -> u64 {
    let program_id = program_id();

    let associated_token_program = ASSOCIATED_TOKEN_PROGRAM_ID.parse::<Pubkey>().unwrap();
    let token_program = TOKEN_PROGRAM_ID;
    let system_program = solana_sdk_ids::system_program::ID;

    let refund_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(s.contributor.pubkey(), true), // contributor
            AccountMeta::new(s.mint, false),                // mint
            AccountMeta::new(s.fundraiser_account, false),  // fundraiser_account
            AccountMeta::new(s.contributor_ata, false),     // contributor_ata ← fixed
            AccountMeta::new(s.contributor_account, false), // contributor_account ← fixed
            AccountMeta::new(s.fundraiser_ata, false),      // fundraiser_ata ← fixed
            AccountMeta::new(token_program, false),
            AccountMeta::new(associated_token_program, false),
            AccountMeta::new(system_program, false),
        ],
        data: vec![3u8],
    };

    let message = Message::new(&[refund_ix], Some(&s.contributor.pubkey()));
    let recent_blockhash = s.svm.latest_blockhash();
    let transaction = Transaction::new(&[&s.contributor], message, recent_blockhash);

    s.svm
        .send_transaction(transaction)
        .unwrap()
        .compute_units_consumed
}

#[test]
fn test_cu() {
    let i = initialize_ix();
    let initialize = i.initialize_cu;

    let mut s = contribute_ix();
    let contribute = s.contribute_cu;

    let refund = do_refund(&mut s);
    let checker = do_checker(&mut s);

    let sep = "+-------------+----------+----------+-------+";
    println!("{sep}");
    println!("| {:<11} | {:>8} ", "instruction", "CU used");
    println!("{sep}");
    println!("| {:<11} | {:>8} ", "initialize", initialize);
    println!("| {:<11} | {:>8} ", "contribute", contribute,);
    println!("| {:<11} | {:>8} ", "checker", checker);
    println!("| {:<11} | {:>8} ", "refund", refund);
    println!("{sep}");
}
