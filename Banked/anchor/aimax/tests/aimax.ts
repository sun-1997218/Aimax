import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AimaxSender } from "../target/types/aimax_sender";
import { AimaxReceiver } from "../target/types/aimax_receiver";

describe("All Tests", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const senderProgram = anchor.workspace.AimaxSender as Program<AimaxSender>;
  const receiverProgram = anchor.workspace.AimaxReceiver as Program<AimaxReceiver>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await senderProgram.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
