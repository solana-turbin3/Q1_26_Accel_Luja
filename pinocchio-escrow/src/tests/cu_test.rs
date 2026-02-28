use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
use solana_keypair::Keypair;
use solana_message::{AccountMeta, Instruction, Message};
use solana_native_token::LAMPORTS_PER_SOL;
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::tests::helper::{make2_ix, make_ix, program_id, Make, TOKEN_PROGRAM_ID};

fn do_take(s: &mut Make, disc: u8) -> u64 {
    let taker = Keypair::new();
    s.svm
        .airdrop(&taker.pubkey(), 10 * LAMPORTS_PER_SOL)
        .unwrap();

    let maker_ata_b = CreateAssociatedTokenAccount::new(&mut s.svm, &taker, &s.mint_b)
        .owner(&s.maker.pubkey())
        .send()
        .unwrap();

    let taker_ata_a = CreateAssociatedTokenAccount::new(&mut s.svm, &taker, &s.mint_a)
        .owner(&taker.pubkey())
        .send()
        .unwrap();
    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut s.svm, &taker, &s.mint_b)
        .owner(&taker.pubkey())
        .send()
        .unwrap();

    MintTo::new(&mut s.svm, &s.maker, &s.mint_b, &taker_ata_b, 1000000000)
        .send()
        .unwrap();

    let take_ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.maker.pubkey(), false),
            AccountMeta::new(taker.pubkey(), true),
            AccountMeta::new(s.escrow_pda, false),
            AccountMeta::new(maker_ata_b, false),
            AccountMeta::new(taker_ata_a, false),
            AccountMeta::new(taker_ata_b, false),
            AccountMeta::new(s.escrow_ata, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![disc],
    };

    let message = Message::new(&[take_ix], Some(&taker.pubkey()));
    let recent_blockhash = s.svm.latest_blockhash();
    let transaction: Transaction = Transaction::new(&[&taker], message, recent_blockhash);
    s.svm
        .send_transaction(transaction)
        .unwrap()
        .compute_units_consumed
}

fn do_refund(s: &mut Make, disc: u8) -> u64 {
    let refund_ix = Instruction {
        program_id: program_id(),
        accounts: vec![
            AccountMeta::new(s.maker.pubkey(), true),
            AccountMeta::new(s.maker_ata_a, false),
            AccountMeta::new(s.escrow_pda, false),
            AccountMeta::new(s.escrow_ata, false),
            AccountMeta::new(TOKEN_PROGRAM_ID, false),
        ],
        data: vec![disc],
    };

    let message = Message::new(&[refund_ix], Some(&s.maker.pubkey()));
    let recent_blockhash = s.svm.latest_blockhash();
    let transaction = Transaction::new(&[&s.maker], message, recent_blockhash);
    s.svm
        .send_transaction(transaction)
        .unwrap()
        .compute_units_consumed
}

#[test]
fn test_cu() {
    let mut s1 = make_ix();
    let make1 = s1.make_cu;
    let take1 = do_take(&mut s1, 1);

    let mut s2 = make2_ix();
    let make2 = s2.make_cu;
    let take2 = do_take(&mut s2, 4);

    let mut s3 = make_ix();
    let refund1 = do_refund(&mut s3, 2);

    let mut s4 = make2_ix();
    let refund2 = do_refund(&mut s4, 5);

    let sep = "+-------------+----------+----------+-------+";
    println!("{sep}");
    println!(
        "| {:<11} | {:>8} | {:>8} | {:>5} |",
        "instruction", "unsafe", "wincode", "diff"
    );
    println!("{sep}");
    println!(
        "| {:<11} | {:>8} | {:>8} | {:>+5} |",
        "make",
        make1,
        make2,
        make2 as i64 - make1 as i64
    );
    println!(
        "| {:<11} | {:>8} | {:>8} | {:>+5} |",
        "take",
        take1,
        take2,
        take2 as i64 - take1 as i64
    );
    println!(
        "| {:<11} | {:>8} | {:>8} | {:>+5} |",
        "cancel",
        refund1,
        refund2,
        refund2 as i64 - refund1 as i64
    );
    println!("{sep}");
}
