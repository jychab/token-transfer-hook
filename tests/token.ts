import * as anchor from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  ExtensionType,
  MAX_FEE_BASIS_POINTS,
  TOKEN_2022_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  createInitializeMintInstruction,
  createInitializeTransferFeeConfigInstruction,
  createInitializeTransferHookInstruction,
  createMintToInstruction,
  createTransferCheckedWithFeeAndTransferHookInstruction,
  getAssociatedTokenAddressSync,
  getMintLen,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { Token } from "../target/types/token";

describe("transfer-hook", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Token as anchor.Program<Token>;

  const wallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;

  // Generate keypair to use as address for the transfer-hook enabled mint
  const mint = new Keypair();
  const decimals = 9;

  // Sender token account address
  const sourceTokenAccount = getAssociatedTokenAddressSync(
    mint.publicKey,
    wallet.publicKey,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // Recipient token account address
  const recipient = new PublicKey(
    "EhbriAJswgCHtZANnv5b1fGYzoe2gvQb72VxP2eo7ULH"
  );
  const destinationTokenAccount = getAssociatedTokenAddressSync(
    mint.publicKey,
    recipient,
    false,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  // ExtraAccountMetaList address
  // Store extra accounts required by the custom transfer hook instruction
  const [extraAccountMetaListPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from("extra-account-metas"), mint.publicKey.toBuffer()],
    program.programId
  );

  const feeBasisPts = 5;
  const maxFee = BigInt(10000000); // max_amount of mint
  const authority = Keypair.generate();

  it("Create Mint Account with Transfer Hook Extension and Permanent Delegate Extension", async () => {
    const extensions = [
      ExtensionType.TransferFeeConfig,
      ExtensionType.TransferHook,
    ];
    const mintLen = getMintLen(extensions);
    const lamports =
      await provider.connection.getMinimumBalanceForRentExemption(mintLen);

    console.log("====================================");
    console.log(authority.publicKey);

    const transaction = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: wallet.publicKey,
        newAccountPubkey: mint.publicKey,
        space: mintLen,
        lamports: lamports,
        programId: TOKEN_2022_PROGRAM_ID,
      }),
      createInitializeTransferFeeConfigInstruction(
        mint.publicKey,
        authority.publicKey,
        authority.publicKey,
        feeBasisPts,
        maxFee,
        TOKEN_2022_PROGRAM_ID
      ),
      createInitializeTransferHookInstruction(
        mint.publicKey,
        wallet.publicKey,
        program.programId, // Transfer Hook Program ID
        TOKEN_2022_PROGRAM_ID
      ),
      createInitializeMintInstruction(
        mint.publicKey,
        decimals,
        wallet.publicKey,
        null,
        TOKEN_2022_PROGRAM_ID
      )
    );

    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer, mint]
    );
    console.log(`Transaction Signature: ${txSig}`);
  });

  // Create the two token accounts for the transfer-hook enabled mint
  // Fund the sender token account with 100 tokens
  it("Create Token Accounts and Mint Tokens", async () => {
    // 100 tokens
    const amount = 100 * 10 ** decimals;

    const transaction = new Transaction().add(
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        sourceTokenAccount,
        wallet.publicKey,
        mint.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createAssociatedTokenAccountInstruction(
        wallet.publicKey,
        destinationTokenAccount,
        recipient,
        mint.publicKey,
        TOKEN_2022_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      createMintToInstruction(
        mint.publicKey,
        sourceTokenAccount,
        wallet.publicKey,
        amount,
        [],
        TOKEN_2022_PROGRAM_ID
      )
    );

    const txSig = await sendAndConfirmTransaction(
      connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true }
    );

    console.log(`Transaction Signature: ${txSig}`);
  });

  it("Create ExtraAccountMetaList Account", async () => {
    const initializeExtraAccountMetaListInstruction = await program.methods
      .initializeExtraAccountMetaList()
      .accounts({
        mint: mint.publicKey,
        extraAccountMetaList: extraAccountMetaListPDA,
      })
      .instruction();

    const transaction = new Transaction().add(
      initializeExtraAccountMetaListInstruction
    );

    const txSig = await sendAndConfirmTransaction(
      provider.connection,
      transaction,
      [wallet.payer],
      {
        skipPreflight: true,
        commitment: "confirmed",
      }
    );
    console.log("Transaction Signature:", txSig);
  });

  it("Transfer Hook with Extra Account Meta", async () => {
    // 1 tokens
    const amount = 1 * 10 ** decimals;
    const bigIntAmount = BigInt(amount);
    const fee =
      (bigIntAmount * BigInt(feeBasisPts)) / BigInt(MAX_FEE_BASIS_POINTS);
    // Determine fee charged
    const feeCharged = fee > maxFee ? maxFee : fee;
    // Standard token transfer instruction
    const transferInstruction =
      await createTransferCheckedWithFeeAndTransferHookInstruction(
        connection,
        sourceTokenAccount,
        mint.publicKey,
        destinationTokenAccount,
        wallet.publicKey,
        bigIntAmount,
        decimals,
        feeCharged,
        [],
        "confirmed",
        TOKEN_2022_PROGRAM_ID
      );

    const transaction = new Transaction().add(transferInstruction);

    const txSig = await sendAndConfirmTransaction(
      connection,
      transaction,
      [wallet.payer],
      { skipPreflight: true }
    );
    console.log("Transfer Signature:", txSig);
  });
});
