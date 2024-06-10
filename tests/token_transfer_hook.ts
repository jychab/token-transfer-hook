import * as anchor from "@coral-xyz/anchor";
import { Token } from "../target/types/token";

describe("token-transfer-hook", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Token as anchor.Program<Token>;
  const wallet = provider.wallet as anchor.Wallet;
  const connection = provider.connection;
});
