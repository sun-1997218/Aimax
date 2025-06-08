use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use solana_program::program::{invoke, invoke_signed};
use solana_program::instruction::Instruction;

pub mod receiver;
use receiver::*;

declare_id!("6DR8jRQALc7Lu6aW6wXdriBv5a7ro8Wcvu33hoTNNgXu");

#[program]
pub mod ray_cross_chain {
    use super::*;


}

