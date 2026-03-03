#[cfg(test)]
mod tests {
    use crate::ID as PROGRAM_ID;
    use litesvm::LiteSVM;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::Transaction,
    };
    use borsh::BorshDeserialize;
    use sha2::{Sha256, Digest};

    #[derive(Debug, BorshDeserialize)]
    struct Poll {
        pub poll_id: u64,
        pub poll_description: String,
        pub poll_start: u64,
        pub poll_end: u64,
        pub candidate_amount: u64,
    }

    #[derive(Debug, BorshDeserialize)]
    struct Candidate {
        pub candidate_name: String,
        pub candidate_vote: u64,
        //pub candidate_votes: u64,
    }

    // Helper function to calculate instruction discriminator
    // This is how Anchor generates discriminators for instructions
    fn get_discriminator(instruction_name: &str) -> [u8; 8] {
        let mut hasher = Sha256::new();
        hasher.update(format!("global:{}", instruction_name));
        let result = hasher.finalize();
        let mut discriminator = [0u8; 8];
        discriminator.copy_from_slice(&result[..8]);
        discriminator
    }

    // poll seed is basically using poll_id. Its the same thing for this example
    fn get_pda(poll_seed: u64, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&poll_seed.to_le_bytes()],
            program_id,
        )
    }

    fn get_candidate_pda(poll_seed: u64, candidate_name: &str, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[&poll_seed.to_le_bytes(), candidate_name.as_bytes()],
            program_id,
        )
    }

    fn initialize_instruction(
        program_id: &Pubkey,
        payer: &Pubkey, 
        poll_id: u64,
        description: String,
        poll_start: u64,
        poll_end: u64,
    ) -> Instruction {
        let (vote_pda, _bump) = get_pda(poll_id, program_id);

        // Build instruction data with proper Borsh serialization
        let discriminator = get_discriminator("initialize");
        let mut instruction_data = Vec::new();
        
        // Add discriminator
        instruction_data.extend_from_slice(&discriminator);

        // Serialize poll_id as u64 (8 bytes, little-endian)
        instruction_data.extend_from_slice(&poll_id.to_le_bytes());

        // Serialize description as String (4 bytes length + UTF-8 bytes)
        let description_bytes = description.as_bytes();
        instruction_data.extend_from_slice(&(description_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(description_bytes);

        // Serialize poll_start as u64 (8 bytes, little-endian)
        instruction_data.extend_from_slice(&poll_start.to_le_bytes());

        // Serialize poll_end as u64 (8 bytes, little-endian)
        instruction_data.extend_from_slice(&poll_end.to_le_bytes());

        Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(vote_pda, false),
                AccountMeta::new(*payer, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        }
        
    }

    fn candidate_instruction(
        program_id: &Pubkey,
        payer: &Pubkey,
        poll_id: u64,
        candidate_name: String,
    ) -> Instruction {
        let (poll_pda, _bump_poll) = get_pda(poll_id, program_id);
        let (candidate_pda, _bump) = get_candidate_pda(poll_id, &candidate_name, program_id);
        // Build instruction data with proper Borsh serialization
        let discriminator = get_discriminator("candidate_initialize");
        let mut instruction_data = Vec::new();

        instruction_data.extend_from_slice(&discriminator);

        // Serialize description as String (4 bytes length + UTF-8 bytes)
        let candidate_name_bytes = candidate_name.as_bytes();
        instruction_data.extend_from_slice(&(candidate_name_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(candidate_name_bytes);

        instruction_data.extend_from_slice(&poll_id.to_le_bytes());

        //instruction_data.extend_from_slice(&candidate_votes.to_le_bytes());

        Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(poll_pda, false),
                AccountMeta::new(candidate_pda, false),
                AccountMeta::new(*payer, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        }

    }

    fn vote(
        program_id: &Pubkey,
        payer: &Pubkey,
        poll_id: u64,
        candidate_name: String,
    ) -> Instruction {
        let (poll_pda, _bump_poll) = get_pda(poll_id, program_id);
        let (candidate_pda, _bump) = get_candidate_pda(poll_id, &candidate_name, program_id);
        // Build instruction data with proper Borsh serialization
        // use the "vote" discriminator (was accidentally reusing candidate_initialize)
        let discriminator = get_discriminator("vote");
        let mut instruction_data = Vec::new();

        instruction_data.extend_from_slice(&discriminator);

        // Serialize candidate name as String (4 bytes length + UTF-8 bytes)
        let candidate_name_bytes = candidate_name.as_bytes();
        instruction_data.extend_from_slice(&(candidate_name_bytes.len() as u32).to_le_bytes());
        instruction_data.extend_from_slice(candidate_name_bytes);

        instruction_data.extend_from_slice(&poll_id.to_le_bytes());

        Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(poll_pda, false),
                AccountMeta::new(candidate_pda, false),
                AccountMeta::new(*payer, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        }

    }

    #[test]
    fn initialize() {
        let mut svm = LiteSVM::new();

        // Use the declared program ID from this crate
        let program_id = PROGRAM_ID;
        let program_bytes = include_bytes!("../../../target/deploy/voting.so");
        svm.add_program(program_id, program_bytes).expect("Failed to add program");

        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap(); // 10 SOL

        let poll_seed: u64 = 1;
        let description = String::from("Who is better? Ronaldo or Messi!");
        let poll_start = 0;
        let poll_end = 1872360440;

        let create_ix = initialize_instruction(&program_id, &payer.pubkey(), poll_seed, description.clone(), poll_start, poll_end);
        //println!("{:?}", create_ix.clone());
        let create_tx = Transaction::new_signed_with_payer(
            &[create_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );

        let tx_result = svm.send_transaction(create_tx);
        assert!(tx_result.is_ok(), "Create transaction failed: {:?}", tx_result.err());


        let (vote_pda, _bump) = get_pda(poll_seed, &program_id);

        // Verify the account was created with correct data
        let account = svm.get_account(&vote_pda).expect("Poll account should exist");

        // Account data starts with 8-byte discriminator
        let poll_data = Poll::deserialize(&mut &account.data[8..]).expect("Failed to deserialize DataStore");
        println!("{:?}", poll_data);

        assert_eq!(poll_data.poll_id, poll_seed, "Id mismatch");
        assert_eq!(poll_data.poll_description, description, "Descriptions mismatch");
        assert_eq!(poll_data.candidate_amount, 0, "Candidate amount should start at zero");
        println!("Create test passed!");
    }

    #[test]
    fn candidate_initialize(){
        let mut svm = LiteSVM::new();

        // Use the declared program ID from this crate
        let program_id = PROGRAM_ID;
        let program_bytes = include_bytes!("../../../target/deploy/voting.so");
        svm.add_program(program_id, program_bytes).expect("Failed to add program");

        let payer = Keypair::new();
        svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap(); // 10 SOL

        // first create a poll so that the candidate init can reference it
        let poll_seed: u64 = 1;
        let description = String::from("Who is better? Ronaldo or Messi!");
        let poll_start = 0;
        let poll_end = 1872360440;

        let create_poll_ix = initialize_instruction(&program_id, &payer.pubkey(), poll_seed, description.clone(), poll_start, poll_end);
        let create_poll_tx = Transaction::new_signed_with_payer(
            &[create_poll_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        let res = svm.send_transaction(create_poll_tx);
        assert!(res.is_ok(), "Poll creation failed: {:?}", res.err());

        // now add first candidate
        let candidate_name_one = "Ronaldo".to_string();
        let create_ix_one = candidate_instruction(&program_id, &payer.pubkey(), poll_seed, candidate_name_one.clone());
        let create_tx_one = Transaction::new_signed_with_payer(
            &[create_ix_one],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        let res = svm.send_transaction(create_tx_one);
        assert!(res.is_ok(), "Candidate1 creation failed: {:?}", res.err());

        let (candidate_pda, _bump) = get_candidate_pda(poll_seed, &candidate_name_one, &program_id);

        // Verify the account was created with correct data
        let account = svm.get_account(&candidate_pda).expect("Candidate account should exist");

        // Account data starts with 8-byte discriminator
        let candidate_one_data = Candidate::deserialize(&mut &account.data[8..]).expect("Failed to deserialize DataStore");
        println!("{:?}", candidate_one_data);

        assert_eq!(candidate_one_data.candidate_name, candidate_name_one, "Candidate name mismatch");
        assert_eq!(candidate_one_data.candidate_vote, 0, "Candidate vote mismatch");


        //THis is test for voting
        let vote_candidate = "Ronaldo".to_string();
        let create_vote_ix = vote(&program_id, &payer.pubkey(), poll_seed, vote_candidate.clone());
        let create_vote_tx = Transaction::new_signed_with_payer(
            &[create_vote_ix],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        let res = svm.send_transaction(create_vote_tx);
        assert!(res.is_ok(), "Voting failed: {:?}", res.err());

        let vote_account = svm.get_account(&candidate_pda).expect("Candidate account should exist");
        let candidate_voted = Candidate::deserialize(&mut &vote_account.data[8..]).expect("Failed to deserialize DataStore");
        // after voting we expect the vote count to increase by 1
        assert_eq!(candidate_voted.candidate_vote, 1, "Candidate vote mismatch");
        println!("voted candidate data: {:?}", candidate_voted);

        // add second candidate as well just to exercise the logic
        let candidate_name_two = String::from("Messi");
        let create_ix_two = candidate_instruction(&program_id, &payer.pubkey(), poll_seed, candidate_name_two.clone());
        let create_tx_two = Transaction::new_signed_with_payer(
            &[create_ix_two],
            Some(&payer.pubkey()),
            &[&payer],
            svm.latest_blockhash(),
        );
        let res = svm.send_transaction(create_tx_two);
        assert!(res.is_ok(), "Candidate2 creation failed: {:?}", res.err());

        let (candidate2_pda, _bump2) = get_candidate_pda(poll_seed, &candidate_name_two, &program_id);
        let account2 = svm.get_account(&candidate2_pda).expect("Second candidate account should exist");
        let candidate_two_data = Candidate::deserialize(&mut &account2.data[8..]).expect("Failed to deserialize DataStore");
        assert_eq!(candidate_two_data.candidate_name, candidate_name_two, "Candidate2 name mismatch");
    }
}
