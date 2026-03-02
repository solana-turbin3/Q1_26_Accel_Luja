pub mod helper;

#[cfg(test)]
mod test {
    use litesvm_token::{CreateAssociatedTokenAccount, MintTo};
    use pinocchio::Address;
    use solana_keypair::Keypair;
    use solana_message::{AccountMeta, Instruction, Message};
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    use crate::tests::helper::{
        ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID, contribute_ix, initialize_ix, program_id,
    };

    #[test]
    pub fn initialize_instruction() {
        initialize_ix();
    }

    #[test]
    pub fn contribute_instruction() {
        contribute_ix();
    }

    #[test]
    pub fn checker_instruction() {
        let mut s = contribute_ix();
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

        let tx = s.svm.send_transaction(transaction).unwrap();
    }

    #[test]
    pub fn refund_instruction() {
        let mut s = contribute_ix();
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

        let tx = s.svm.send_transaction(transaction).unwrap();
    }
}
