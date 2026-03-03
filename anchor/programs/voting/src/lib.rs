use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[cfg(test)]
mod tests;

declare_id!("FnjZB6c8Z42YprQmZKQ2z72naCoXYPUqUue6D8TgWmRK");

#[program]
pub mod voting {
    use super::*;

    pub fn initialize(
        ctx: Context<InitializePoll>, 
        poll_id: u64, 
        description: String,
        poll_start: u64,
        poll_end: u64,
    ) -> Result<()> {
        let poll = &mut ctx.accounts.poll;
        poll.poll_id = poll_id;
        poll.poll_description = description;
        poll.poll_start = poll_start;
        poll.poll_end = poll_end;
        poll.candidate_amount = 0;
        Ok(())
    }

    pub fn candidate_initialize(
        ctx: Context<CandidateInitialize>, 
        candidate_name: String, 
        _poll_id: u64
    ) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate;
        let poll = &mut ctx.accounts.poll;
        poll.candidate_amount += 1;
        candidate.candidate_name = candidate_name;
        candidate.candidate_vote = 0;
        Ok(())
    }

    pub fn vote(ctx: Context<VoteAccounts>, candidate_name: String, _poll_id: u64) -> Result<()> {
        let candidate = &mut ctx.accounts.candidate;
        candidate.candidate_vote += 1;
        Ok(())
    }
    
}

#[derive(Accounts)]
#[instruction(poll_id: u64)]
pub struct InitializePoll<'info> {
    #[account(
        init,
        payer = signer,
        space = 8 + Poll::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll : Account<'info, Poll>,
    #[account(mut)]
    pub signer : Signer<'info>,
    pub system_program : Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct CandidateInitialize<'info> {
    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll : Account<'info, Poll>,

    #[account(
        init,
        payer = signer,
        space = 8 + Candidate::INIT_SPACE,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
        bump,
    )]
    pub candidate : Account<'info, Candidate>,
    #[account(mut)]
    pub signer : Signer<'info>,
    pub system_program : Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(candidate_name: String, poll_id: u64)]
pub struct VoteAccounts<'info> {
    #[account(
        seeds = [poll_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub poll : Account<'info, Poll>,

    #[account(
        mut,
        seeds = [poll_id.to_le_bytes().as_ref(), candidate_name.as_bytes()],
        bump,
    )]
    pub candidate : Account<'info, Candidate>,
    #[account(mut)]
    pub signer : Signer<'info>,
}




#[account]
#[derive(InitSpace)]
pub struct Poll {
    pub poll_id: u64,
    #[max_len(200)]
    pub poll_description: String,
    pub poll_start: u64,
    pub poll_end: u64,
    pub candidate_amount: u64,
}

#[account]
#[derive(InitSpace)]
pub struct Candidate {
    #[max_len(40)]
    pub candidate_name: String,
    pub candidate_vote: u64,
    
}