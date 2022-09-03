import * as anchor from "@project-serum/anchor";

export const getTransactionFees = async (
  connection: anchor.web3.Connection,
  transaction: anchor.web3.Transaction,
  provider: anchor.web3.PublicKey
) => {
  const { blockhash } = await connection.getLatestBlockhash();

  transaction.recentBlockhash = blockhash;
  transaction.feePayer = provider;
  const { value: fees } = await connection.getFeeForMessage(
    transaction.compileMessage()
  );

  return fees;
};
