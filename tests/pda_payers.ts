import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PdaPayers } from "../target/types/pda_payers";

describe("pda_payers", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PdaPayers as Program<PdaPayers>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
