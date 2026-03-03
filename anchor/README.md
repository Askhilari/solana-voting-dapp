# Anchor Vault Program

This template includes a simple SOL voting program built with [Anchor](https://www.anchor-lang.com/) and [LiteSVM] (https://www.litesvm.com/)

## Running the Program

The voting program can be tested on the cli itself. In the lib.rs folder you have all the instructions and accounts that you need to Vote.
The initialize sets up the poll with all poll parameters, candidate_initialize lists the candidate name and votes and the vote account increments vote by 1 everytime we select the candidate name.

## Interacting with program

```bash
anchor build
```
This command builds the program and gives errors that can occur when building the program. Once the build succeeds we can test the instructions using liteSVM. The way we use these tests are. We have a struct (similar to #[accounts] data from lib.rs), then we need a get discriminator which takes the inputs of all our values and converts to bytes and outputs u8 array with bytes.

Then we need to find the program derived address (PDA) of the program. This is the assigned to our program so that the state is saved and we don't reset the program everytime we initialize. To find the PDA we need a seed and program id. The program id is same for both, but for poll account we use poll_id as our seed and for candidate account we use poll_id + candidate_name as our seed. The inputes are converted to bytes and passed.

When using liteSVM we need instructions when testing our program. In the Instruction we pass the functions like inititalize, candidate_initialize and vote which we need to test. Calling instructions is similar for all cases. We get the PDA using our find_program_address fuctions from above. Then we call discriminator and we create instruction_data vec which takes the discriminator + poll_id bytes + poll_id length + description bytes + description lenght + ... . Here we are basically appending to instruction_data vec the parameters that our function takes as input.

Once everything is appended, we can put it in Instruction:
Instruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(vote_pda, false),
                AccountMeta::new(*payer, true),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: instruction_data,
        }
This is our output, Instruction has 3 parameters, program id, accounts (This takes our pda, payer and system_program) we pass after each account whether it is signer (true or false) and it takes out instruction_data vec for our data parameter.


Then under #[test] we call the behaviour that we need to test. We use deserialize fucntion to get the actual inputs and upon tested candidate_initialize we can see that vote feature works and gets incremented.

## Testing the functions

Install dependencies

```bash
cargo add litesvm
cargo add borsh
cargo add sha2
```

Running tests

```bash
cargo test
```

Or if you want to test individual tests you can do!

```bash
cargo test candidate_initialize -- --nocapture
```

-- --nocapture lets you display all the println! statements from code.

If you get payer error you will have to generate a payer keypair.

```bash
cd anchor
solana-keygen new -o target/deploy/voting-keypair.json
```

```bash
solana address -k target/deploy/voting-keypair.json
```

This program is a basic demonstration of voting dapp using solana and it lets you test the functions that you will deploy and attach to the frontend.