import * as anchor from "@coral-xyz/anchor";
import { TokenTransferHook } from "../target/types/token_transfer_hook";

describe("token-transfer-hook", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .TokenTransferHook as anchor.Program<TokenTransferHook>;
  const wallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;
});
