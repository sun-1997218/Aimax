import * as anchor from '@project-serum/anchor';
import { Program, BN } from '@project-serum/anchor';
import { AimaxReceiver } from '../target/types/aimax_receiver';
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { Token, TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { assert } from 'chai';

describe('aimax_receiver', () => {
  const provider = anchor.Provider.env();
  anchor.setProvider(provider);
  const wallet = provider.wallet;
  const program = anchor.workspace.AimaxReceiver as Program<AimaxReceiver>;

  // 配置官方地址 [1,6](@ref)
  const OFFICIAL_ROUTER = new PublicKey('Ccip842gzYHhvdDkSyi2YVCoAWPbYJoApMFzSxQroE9C');
  const OFFICIAL_OFFRAMP = new PublicKey('offqSMQWgQud6WJz694LRzkeN5kMYpCHTpXQr3Rkcjm');

  // 账户声明
  let state: PublicKey;
  let messagesStorage: PublicKey;
  let tokenAdmin: PublicKey;
  let allowedOfframp: PublicKey;
  let authority: PublicKey;
  let tokenMint: Token;
  let programTokenAccount: PublicKey;

  // 初始化测试环境
  before(async () => {
    // 生成PDA账户
    [state] = await PublicKey.findProgramAddress(
      [Buffer.from('state')],
      program.programId
    );
    [messagesStorage] = await PublicKey.findProgramAddress(
      [Buffer.from('messages_storage')],
      program.programId
    );
    [tokenAdmin] = await PublicKey.findProgramAddress(
      [Buffer.from('token_admin')],
      program.programId
    );
    
    // 计算allowedOfframp PDA (必须与Router程序匹配) [6](@ref)
    [allowedOfframp] = await PublicKey.findProgramAddress(
      [Buffer.from('allowed_offramp'), OFFICIAL_ROUTER.toBuffer()],
      program.programId
    );
    
    // 生成authority PDA (需匹配OffRamp的链标识符) [1,6](@ref)
    [authority] = await PublicKey.findProgramAddress(
      [
        Buffer.from('external_execution_config'),
        // 此处的链标识符需与OffRamp实际使用的匹配（示例为测试值）
        Buffer.from('c68_5faff46d0407f409272efa613e1968f5480fa3803e7fbbfc747c2a82c5d6'),
        OFFICIAL_OFFRAMP.toBuffer()
      ],
      program.programId
    );

    // 创建代币
    tokenMint = await Token.createMint(
      provider.connection,
      wallet.payer,
      wallet.publicKey,
      null,
      9,
      TOKEN_PROGRAM_ID
    );

    // 创建程序代币账户
    programTokenAccount = await tokenMint.createAccount(tokenAdmin);
    await tokenMint.mintTo(programTokenAccount, wallet.payer, [], 1_000_000_000);
  });

  // 测试1：接收CCIP消息（使用官方地址）
  it('通过官方OffRamp接收消息', async () => {
    const message = {
      messageId: new Uint8Array(32).fill(1),
      sourceChainSelector: new BN(1),
      sender: Buffer.from('sender_address'),
      data: Buffer.from('Hello, CCIP!'),
      tokenAmounts: [{
        token: tokenMint.publicKey,
        amount: new BN(100_000_000) // 0.1代币
      }]
    };

    await program.methods.ccipReceive(message)
      .accounts({
        authority, // 使用官方OffRamp生成的PDA
        offrampProgram: OFFICIAL_OFFRAMP, // 官方OffRamp地址
        allowedOfframp, // Router生成的验证PDA
        state,
        messagesStorage,
        tokenAdmin
      })
      .remainingAccounts([ // 代币转移所需账户
        { pubkey: programTokenAccount, isWritable: true, isSigner: false },
        { pubkey: tokenMint.publicKey, isWritable: false, isSigner: false },
        { pubkey: TOKEN_PROGRAM_ID, isWritable: false, isSigner: false }
      ])
      .rpc({ skipPreflight: false }); // 确保交易上链

    // 验证消息存储
    const storage = await program.account.messagesStorage.fetch(messagesStorage);
    assert.equal(storage.latestMessage.data.toString(), 'Hello, CCIP!');
  });

  // 测试2：安全边界验证
  it('拒绝未授权的OffRamp调用', async () => {
    const fakeOfframp = Keypair.generate(); // 未授权地址
    
    try {
      await program.methods.ccipReceive({...})
        .accounts({ 
          offrampProgram: fakeOfframp.publicKey, // 恶意地址
          // 其他账户...
        })
        .rpc();
      assert.fail('应拒绝未授权调用');
    } catch (err) {
      assert.include(err.logs.toString(), 'Unauthorized');
    }
  });
});
