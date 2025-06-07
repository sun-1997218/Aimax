use anchor_lang::prelude::*;

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
