use anchor_lang::prelude::*;
use solana_program::{program_error,entrypoint::ProgramResult};

declare_id!("6DR8jRQALc7Lu6aW6wXdriBv5a7ro8Wcvu33hoTNNgXu");

#[program]
pub mod aimax {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
//标准的CCIP消息体
pub struct SVM2AnyMessage{
    pub receiver : Vec<u8>,
    pub data:Vec<u8>,
    pub token_amounts:Vec<SVM2AnyMessage>,
    pub fee_token:Pubkey,
    pub extra_args:Vec<u8>,
}