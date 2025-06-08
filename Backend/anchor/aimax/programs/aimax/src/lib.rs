use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use solana_program::program::{invoke, invoke_signed};
use solana_program::instruction::Instruction;

declare_id!("6DR8jRQALc7Lu6aW6wXdriBv5a7ro8Wcvu33hoTNNgXu");

#[program]
pub mod ray_cross_chain {
    use super::*;

    /// 初始化配置账户
    pub fn initialize(ctx: Context<Initialize>, fee_amount: u64) -> Result<()> {
        let config = &mut ctx.accounts.config;
        config.owner = *ctx.accounts.user.key;
        config.fee_amount = fee_amount;
        config.ray_token = ctx.accounts.ray_token.key();
        Ok(())
    }

    /// 发送 RAY 到 Ethereum
    pub fn send_ray_cross_chain(
        ctx: Context<SendRayCrossChain>,
        destination_chain: String,
        receiver: String, // Ethereum 地址（如 "0x1234..."）
        amount: u64,
    ) -> Result<()> {
        let chain_selector = match destination_chain.as_str() {
            "Ethereum" => 5009297550715157269_u64, // Ethereum Mainnet chain selector
            _ => return Err(ErrorCode::InvalidChain.into()),
        };

        // 构造 CCIP 消息
        let message = SVM2AnyMessage {
            receiver: hex::decode(receiver.trim_start_matches("0x"))
                .map_err(|_| ErrorCode::InvalidCCIPData)?,
            data: vec![], // 可选自定义数据
            token_amounts: vec![TokenAmount {
                token: ctx.accounts.config.ray_token,
                amount,
            }],
            fee_token: ctx.accounts.link_token_account.mint,
            extra_args: vec![],
        };

        // 序列化消息
        let message_data = message.try_to_vec()?;

        // 构造 CCIP 指令
        let ccip_program_id = Pubkey::from("CCIP2sQ8xwp4R1vSzV4Qe69p6W82xpenczRVfTjrA8yc")?;
        let ccip_instruction = Instruction {
            program_id: ccip_program_id,
            accounts: vec![
                AccountMeta::new(ctx.accounts.ccip_account.key(), false),
                AccountMeta::new(ctx.accounts.user.key(), true),
                AccountMeta::new(ctx.accounts.ray_token_account.key(), false),
                AccountMeta::new(ctx.accounts.ccip_pool.key(), false),
                AccountMeta::new_readonly(ctx.accounts.token_program.key(), false),
                AccountMeta::new(ctx.accounts.ccip_fee_account.key(), false),
            ],
            data: message_data,
        };

        // 执行 CPI 调用
        invoke(
            &ccip_instruction,
            &[
                ctx.accounts.ccip_account.to_account_info(),
                ctx.accounts.user.to_account_info(),
                ctx.accounts.ray_token_account.to_account_info(),
                ctx.accounts.ccip_pool.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.ccip_fee_account.to_account_info(),
            ],
        )?;

        // 支付 LINK 费用
        let transfer_link_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.link_token_account.to_account_info(),
                to: ctx.accounts.ccip_fee_account.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_link_cpi, ctx.accounts.config.fee_amount)?;

        // 转移 RAY 到 CCIP 资金池
        let transfer_ray_cpi = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.ray_token_account.to_account_info(),
                to: ctx.accounts.ccip_pool.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        );
        token::transfer(transfer_ray_cpi, amount)?;

        emit!(RayTransferSent {
            destination_chain,
            receiver,
            amount,
        });
        Ok(())
    }

    /// 接收 CCIP 消息（从 Ethereum 到 Solana）
    pub fn ccip_receive(ctx: Context<CcipReceive>, message: SVM2AnyMessage) -> Result<()> {
        // 验证消息来源（由 CCIP 程序调用）
        require!(
            ctx.accounts.ccip_account.key() == Pubkey::from_str("CCIP2sQ8xwp4R1vSzV4Qe69p6W82xpenczRVfTjrA8yc")?,
            ErrorCode::InvalidCCIPProgram
        );

        // 解析消息
        let receiver = ctx.accounts.receiver.key();
        let token_amounts = message.token_amounts;
        let amount = token_amounts
            .iter()
            .find(|t| t.token == ctx.accounts.config.ray_token)
            .map(|t| t.amount)
            .ok_or(ErrorCode::InvalidToken)?;

        // 铸造 RAY 代币到接收者账户
        let cpi_accounts = token::MintTo {
            mint: ctx.accounts.ray_mint.to_account_info(),
            to: ctx.accounts.receiver_token_account.to_account_info(),
            authority: ctx.accounts.mint_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, amount)?;

        emit!(RayReceived {
            receiver,
            amount,
            source_chain: "Ethereum".to_string(), // 假设来自 Ethereum
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user, space = 8 + 32 + 8 + 32, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub ray_token: Account<'info, Mint>, // RAY 代币的 Mint 账户
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SendRayCrossChain<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, has_one = owner @ ErrorCode::UnauthorizedUser)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub ray_token_account: Account<'info, TokenAccount>, // 用户的 RAY 代币账户
    #[account(mut)]
    pub link_token_account: Account<'info, TokenAccount>, // 用户的 LINK 代币账户
    #[account(mut)]
    pub ccip_account: AccountInfo<'info>, // CCIP 程序账户
    #[account(mut)]
    pub ccip_pool: AccountInfo<'info>, // CCIP 资金池账户
    #[account(mut)]
    pub ccip_fee_account: AccountInfo<'info>, // CCIP 费用接收账户
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct CcipReceive<'info> {
    #[account(mut)]
    pub config: Account<'info, Config>,
    #[account(mut)]
    pub receiver: AccountInfo<'info>, // 接收者账户
    #[account(mut)]
    pub receiver_token_account: Account<'info, TokenAccount>, // 接收者的 RAY 代币账户
    #[account(mut)]
    pub ray_mint: Account<'info, Mint>, // RAY Mint 账户
    #[account(mut)]
    pub mint_authority: AccountInfo<'info>, // Mint 权限账户（需配置）
    pub ccip_account: AccountInfo<'info>, // CCIP 程序账户
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Config {
    pub owner: Pubkey,
    pub fee_amount: u64, // LINK 费用
    pub ray_token: Pubkey, // RAY 代币地址
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Debug)]
pub struct SVM2AnyMessage {
    pub receiver: Vec<u8>, // EVM 地址（20 字节）
    pub data: Vec<u8>,     // 自定义数据
    pub token_amounts: Vec<TokenAmount>, // 代币转移数组
    pub fee_token: Pubkey, // 费用代币（LINK）
    pub extra_args: Vec<u8>, // 额外参数
}

#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq, Debug)]
pub struct TokenAmount {
    pub token: Pubkey, // 代币地址（如 RAY）
    pub amount: u64,   // 转移数量
}

#[event]
pub struct RayTransferSent {
    pub destination_chain: String,
    pub receiver: String,
    pub amount: u64,
}

#[event]
pub struct RayReceived {
    pub receiver: Pubkey,
    pub amount: u64,
    pub source_chain: String,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid destination chain")]
    InvalidChain,
    #[msg("Unauthorized user")]
    UnauthorizedUser,
    #[msg("Invalid CCIP data")]
    InvalidCCIPData,
    #[msg("Invalid CCIP program")]
    InvalidCCIPProgram,
    #[msg("Invalid token")]
    InvalidToken,
}