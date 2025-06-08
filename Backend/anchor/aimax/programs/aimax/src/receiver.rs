use anchor_lang::prelude::*;
use anchor_spl::token::{Mint,TokenAccount};
use anchor_spl::token_2022::spl_token_2022::{self,instruction::transfer_checked};
use solana_program::program::invoke_signed;

//生成seed和bump，实现不同的PDA地址,便于验证地址的唯一性
pub const EXTERNAL_EXECUTION_CONFIG_SEED: &[u8] = b"external_execution_config";
pub const APPROVED_SENDER_SEED: &[u8] = b"approved_ccip_sender";
pub const TOKEN_ADMIN_SEED: &[u8] = b"receiver_token_admin";
pub const ALLOWED_OFFRAMP: &[u8] = b"allowed_offramp"; //允许
pub const STATE: &[u8] = b"state";

