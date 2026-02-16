#[cfg(test)]
mod test {
    use {
        anchor_lang::{prelude::*, InstructionData, ToAccountMetas},
        litesvm::LiteSVM,
        litesvm_token::CreateAssociatedTokenAccount,
        solana_instruction::Instruction,
        solana_keypair::Keypair,
        solana_message::Message,
        solana_native_token::LAMPORTS_PER_SOL,
        solana_pubkey::Pubkey,
        solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM_ID,
        solana_signer::Signer,
        solana_transaction::Transaction,
        spl_token_2022::ID as TOKEN_2022_PROGRAM_ID,
        std::path::PathBuf,
    };

    use crate::{constant::*, state::User};

    static PROGRAM_ID: Pubkey = crate::ID;

    fn setup() -> (LiteSVM, Keypair) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new();

        svm.airdrop(&payer.pubkey(), 1000 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop");

        let program_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/transfer_hook_vault.so");

        let program_data = std::fs::read(&program_path).expect("Failed to read program file");
        svm.add_program(PROGRAM_ID, &program_data);

        (svm, payer)
    }

    #[test]
    fn test_initialize() {
        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();
        let (vault_pda, vault_bump) =
            Pubkey::find_program_address(&[VAULT_SEED, admin.key().as_ref()], &PROGRAM_ID);

        let initialize_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Initialize {
                admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Initialize {
                fee: 50,
                decimal: 6,
            }
            .data(),
        };

        let message = Message::new(&[initialize_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, recent_blockhash);

        let result = svm
            .send_transaction(tx)
            .expect("Transaction should succeed");
        msg!("Initialized");
    }

    #[test]
    fn test_initialize_transfer_hook() {
        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();
        let (vault_pda, vault_bump) =
            Pubkey::find_program_address(&[VAULT_SEED, admin.key().as_ref()], &PROGRAM_ID);

        let initialize_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Initialize {
                admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Initialize {
                fee: 50,
                decimal: 6,
            }
            .data(),
        };

        let message = Message::new(&[initialize_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, recent_blockhash);

        let result = match svm.send_transaction(tx) {
            Ok(r) => r,
            Err(e) => panic!("Tx failed"),
        };
        msg!("Initialized");

        let (extra_meta_pda, _) =
            Pubkey::find_program_address(&[EXTRA_META_SEED, mint.pubkey().as_ref()], &PROGRAM_ID);

        let init_transfer_hook_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::InitializeExtraAccountMetaList {
                payer: admin,
                extra_account_meta_list: extra_meta_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::InitializeTransferHook {}.data(),
        };

        let message = Message::new(&[init_transfer_hook_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, recent_blockhash);

        let result = match svm.send_transaction(tx) {
            Ok(r) => r,
            Err(e) => panic!("Tx failed"),
        };
        msg!("Initialized metadata account");
    }

    #[test]
    fn test_add_remove_user() {
        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();
        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();
        let (vault_pda, vault_bump) =
            Pubkey::find_program_address(&[VAULT_SEED, admin.key().as_ref()], &PROGRAM_ID);

        let initialize_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Initialize {
                admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                system_program: SYSTEM_PROGRAM_ID,
                token_program: TOKEN_2022_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Initialize {
                fee: 50,
                decimal: 6,
            }
            .data(),
        };

        let message = Message::new(&[initialize_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, recent_blockhash);

        let result = match svm.send_transaction(tx) {
            Ok(r) => r,
            Err(e) => panic!("Tx failed"),
        };
        msg!("Initialized");

        let (user_pda, user_bump) =
            Pubkey::find_program_address(&[USER_SEED, user.pubkey().as_ref()], &PROGRAM_ID);

        let add_user_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::AddUser {
                admin,
                user_info: user_pda,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::AddUser {
                user: user.pubkey(),
            }
            .data(),
        };
        let message = Message::new(&[add_user_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, recent_blockhash);
        let result = match svm.send_transaction(tx) {
            Ok(r) => r,
            Err(e) => panic!("Tx failed"),
        };

        let user_account = svm.get_account(&user_pda).unwrap();
        let user_data = User::try_deserialize(&mut user_account.data.as_ref()).unwrap();
        assert_eq!(user_data.address, user.pubkey());
        assert_eq!(user_data.bump, user_bump);

        msg!("New user added");

        let remove_user_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::RemoveUser {
                admin,
                user_info: user_pda,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::RemoveUser {
                user: user.pubkey(),
            }
            .data(),
        };
        let message = Message::new(&[remove_user_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, recent_blockhash);
        let result = match svm.send_transaction(tx) {
            Ok(r) => r,
            Err(e) => panic!("Tx failed"),
        };

        let user_account_after = svm.get_account(&user_pda).unwrap();
        assert_eq!(user_account_after.lamports, 0);
        msg!("User removed");
    }

    #[test]
    fn test_deposit_and_withdraw() {
        let (mut svm, payer) = setup();
        let admin = payer.pubkey();
        let mint = Keypair::new();
        let user = Keypair::new();
        svm.airdrop(&user.pubkey(), 10 * LAMPORTS_PER_SOL).unwrap();

        let (vault_pda, _vault_bump) =
            Pubkey::find_program_address(&[VAULT_SEED, admin.as_ref()], &PROGRAM_ID);

        // Initialize
        let initialize_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Initialize {
                admin,
                vault: vault_pda,
                mint: mint.pubkey(),
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Initialize {
                fee: 50,
                decimal: 6,
            }
            .data(),
        };

        let message = Message::new(&[initialize_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer, &mint], message, recent_blockhash);
        svm.send_transaction(tx).expect("Initialize failed");
        println!("✅ Initialized");

        // Add user
        let (user_pda, user_bump) =
            Pubkey::find_program_address(&[USER_SEED, user.pubkey().as_ref()], &PROGRAM_ID);

        let add_user_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::AddUser {
                admin,
                user_info: user_pda,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::AddUser {
                user: user.pubkey(),
            }
            .data(),
        };

        let message = Message::new(&[add_user_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&payer], message, recent_blockhash);
        svm.send_transaction(tx).expect("Add user failed");

        let user_account = svm.get_account(&user_pda).expect("User not found");
        let user_data = User::try_deserialize(&mut user_account.data.as_ref()).unwrap();
        assert_eq!(user_data.address, user.pubkey());
        assert_eq!(user_data.bump, user_bump);
        println!("✅ New user added");

        // Create ATA
        let user_ata = CreateAssociatedTokenAccount::new(&mut svm, &payer, &mint.pubkey())
            .owner(&user.pubkey())
            .token_program_id(&TOKEN_2022_PROGRAM_ID)
            .send()
            .expect("Failed to create ATA");

        println!("✅ Created user ATA: {}", user_ata);

        // Deposit
        let deposit_amount = 5 * LAMPORTS_PER_SOL;
        let vault_before_balance = svm.get_balance(&vault_pda).unwrap_or(0);
        let user_before_balance = svm.get_balance(&user.pubkey()).unwrap();

        println!("User balance before deposit: {}", user_before_balance);
        println!("Vault balance before deposit: {}", vault_before_balance);

        let deposit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Deposit {
                user: user.pubkey(),
                vault: vault_pda,
                user_info: user_pda,
                user_token_ata: user_ata,
                mint: mint.pubkey(),
                associated_token_program: anchor_spl::associated_token::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Deposit {
                amount: deposit_amount,
            }
            .data(),
        };

        let message = Message::new(&[deposit_ix], Some(&user.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&user], message, recent_blockhash);

        let result = svm.send_transaction(tx);
        match result {
            Ok(_) => println!("✅ Deposit successful"),
            Err(e) => {
                eprintln!("❌ Deposit failed!");
                eprintln!("Error: {:#?}", e);
                panic!("Deposit transaction failed: {:#?}", e);
            }
        }

        let vault_after_balance = svm.get_balance(&vault_pda).unwrap();
        println!("Vault balance after deposit: {}", vault_after_balance);
        assert_eq!(vault_after_balance - vault_before_balance, deposit_amount);

        // Withdraw
        let withdraw_amount = 2 * LAMPORTS_PER_SOL;
        let vault_before_withdraw = svm.get_balance(&vault_pda).unwrap();

        let withdraw_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Withdraw {
                user: user.pubkey(),
                vault: vault_pda,
                user_info: user_pda,
                user_token_ata: user_ata,
                mint: mint.pubkey(),
                associated_token_program: anchor_spl::associated_token::ID,
                token_program: TOKEN_2022_PROGRAM_ID,
                system_program: SYSTEM_PROGRAM_ID,
            }
            .to_account_metas(None),
            data: crate::instruction::Withdraw {
                amount: withdraw_amount,
            }
            .data(),
        };

        let message = Message::new(&[withdraw_ix], Some(&user.pubkey()));
        let recent_blockhash = svm.latest_blockhash();
        let tx = Transaction::new(&[&user], message, recent_blockhash);

        let result = svm.send_transaction(tx);
        match result {
            Ok(_) => println!("✅ Withdraw successful"),
            Err(e) => {
                eprintln!("❌ Withdraw failed!");
                eprintln!("Error: {:#?}", e);
                panic!("Withdraw transaction failed: {:#?}", e);
            }
        }

        let vault_after_withdraw = svm.get_balance(&vault_pda).unwrap();
        assert_eq!(
            vault_before_withdraw - vault_after_withdraw,
            withdraw_amount
        );
    }
}
