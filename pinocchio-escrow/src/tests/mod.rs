#[cfg(test)]
mod helper;
#[cfg(test)]
mod test {
    use std::vec;

    use crate::{
        instructions::take,
        tests::helper::{
            make_ix, program_id, setup, ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID,
        },
    };
    use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
    use solana_keypair::Keypair;
    use solana_message::{AccountMeta, Instruction, Message};
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    #[test]
    pub fn make_instruction() {
        make_ix();
    }

    #[test]
    pub fn take_instruction() {
        let taker = Keypair::new();
        let mut s = make_ix();

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
            data: vec![1u8],
        };

        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = s.svm.latest_blockhash();
        let transaction: Transaction = Transaction::new(&[&taker], message, recent_blockhash);
        let tx = s.svm.send_transaction(transaction).unwrap();

        // println!("Tx completed:{:?}", tx);
    }

    #[test]
    pub fn refund_instruction() {
        let mut s = make_ix();

        let refund_ix = Instruction {
            program_id: program_id(),
            accounts: vec![
                AccountMeta::new(s.maker.pubkey(), true),
                AccountMeta::new(s.maker_ata_a, false),
                AccountMeta::new(s.escrow_pda, false),
                AccountMeta::new(s.escrow_ata, false),
                AccountMeta::new(TOKEN_PROGRAM_ID, false),
            ],
            data: vec![2u8],
        };

        let message = Message::new(&[refund_ix], Some(&s.maker.pubkey()));
        let recent_blockhash = s.svm.latest_blockhash();
        let transaction = Transaction::new(&[&s.maker], message, recent_blockhash);
        let tx = s.svm.send_transaction(transaction);

        println!("Tx completed:{:?}", tx);
    }
}
