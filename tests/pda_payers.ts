import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { BN } from "bn.js";
import { assert } from "chai";
import { PdaPayers } from "../target/types/pda_payers";
import { getTransactionFees } from "./utils";

describe("pda_payers", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.PdaPayers as Program<PdaPayers>;
  const feeVaultBalance = anchor.web3.LAMPORTS_PER_SOL;
  const aliceKeypair = anchor.web3.Keypair.generate();
  let feeVaultPublicKey: anchor.web3.PublicKey;
  let feeVaultWalletPublicKey: anchor.web3.PublicKey;
  let aliceCollaboratorPublicKey: anchor.web3.PublicKey;
  let collaboratorRentExemption: number;

  before(async () => {
    [feeVaultPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("fee_vault", "utf-8"),
        program.provider.publicKey.toBuffer(),
      ],
      program.programId
    );
    [feeVaultWalletPublicKey] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("fee_vault_wallet", "utf-8"), feeVaultPublicKey.toBuffer()],
      program.programId
    );
    [aliceCollaboratorPublicKey] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from("collaborator", "utf-8"),
          feeVaultPublicKey.toBuffer(),
          aliceKeypair.publicKey.toBuffer(),
        ],
        program.programId
      );
    collaboratorRentExemption =
      await program.provider.connection.getMinimumBalanceForRentExemption(9);
  });

  it("should create a fee vault", async () => {
    // act
    await program.methods
      .createFeeVault(new BN(feeVaultBalance))
      .accounts({
        authority: program.provider.publicKey,
      })
      .rpc();
    // assert
    const feeVaultAccount = await program.account.feeVault.fetch(
      feeVaultPublicKey
    );
    const feeVaultWalletAccount =
      await program.provider.connection.getAccountInfo(feeVaultWalletPublicKey);
    assert.isDefined(feeVaultAccount);
    assert.isDefined(feeVaultWalletAccount);
    assert.equal(feeVaultWalletAccount.lamports, feeVaultBalance);
  });

  it("should deposit to fee vault", async () => {
    // act
    await program.methods
      .depositInFeeVault(new BN(feeVaultBalance))
      .accounts({
        authority: program.provider.publicKey,
      })
      .rpc();
    // assert
    const feeVaultWalletAccount =
      await program.provider.connection.getAccountInfo(feeVaultWalletPublicKey);
    assert.isDefined(feeVaultWalletAccount);
    assert.equal(feeVaultWalletAccount.lamports, feeVaultBalance * 2);
  });

  it("should withdraw from fee vault", async () => {
    // act
    await program.methods
      .withdrawFromFeeVault(new BN(feeVaultBalance))
      .accounts({
        authority: program.provider.publicKey,
      })
      .rpc();
    // assert
    const feeVaultWalletAccount =
      await program.provider.connection.getAccountInfo(feeVaultWalletPublicKey);
    assert.isDefined(feeVaultWalletAccount);
    assert.equal(feeVaultWalletAccount.lamports, feeVaultBalance);
  });

  it("should create collaborator using fee vault", async () => {
    // arrange
    const fees = await getTransactionFees(
      program.provider.connection,
      await program.methods
        .createCollaborator()
        .accounts({
          authority: program.provider.publicKey,
          collaboratorBase: aliceKeypair.publicKey,
        })
        .transaction(),
      program.provider.publicKey
    );
    const beforeAuthorityAccount =
      await program.provider.connection.getAccountInfo(
        program.provider.publicKey
      );
    // act
    await program.methods
      .createCollaborator()
      .accounts({
        authority: program.provider.publicKey,
        collaboratorBase: aliceKeypair.publicKey,
      })
      .rpc();
    // assert
    const aliceCollaboratorAccount = await program.account.collaborator.fetch(
      aliceCollaboratorPublicKey
    );
    const feeVaultWalletAccount =
      await program.provider.connection.getAccountInfo(feeVaultWalletPublicKey);
    const afterAuthorityAccount =
      await program.provider.connection.getAccountInfo(
        program.provider.publicKey
      );
    assert.isDefined(aliceCollaboratorAccount);
    assert.isDefined(feeVaultWalletAccount);
    assert.equal(
      feeVaultWalletAccount.lamports,
      feeVaultBalance - collaboratorRentExemption
    );
    assert.isDefined(beforeAuthorityAccount);
    assert.isDefined(afterAuthorityAccount);
    assert.equal(beforeAuthorityAccount.lamports - fees, afterAuthorityAccount.lamports)
  });
});
