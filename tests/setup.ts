import { PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { MonacoProtocol } from "../target/types/monaco_protocol";
import {
  authoriseAdminOperator,
  authoriseOperator,
  getOrCreateMarketType,
  createProtocolProduct,
  OperatorType,
} from "./util/test_util";

module.exports = async function (_globalConfig, _projectConfig) {
  try {
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);
    const protocolProgram: anchor.Program<MonacoProtocol> =
      anchor.workspace.MonacoProtocol;
    const operatorPk: PublicKey = provider.wallet.publicKey;
    await authoriseAdminOperator(operatorPk, protocolProgram, provider);
    await authoriseOperator(
      operatorPk,
      protocolProgram,
      provider,
      OperatorType.MARKET,
    );
    await authoriseOperator(
      operatorPk,
      protocolProgram,
      provider,
      OperatorType.CRANK,
    );

    await createProtocolProduct(provider);

    await getOrCreateMarketType(
      protocolProgram as anchor.Program,
      "EventResultWinner",
    );
  } catch (err) {
    console.error(err);
    throw err;
  }
};
