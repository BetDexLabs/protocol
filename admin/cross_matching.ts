import { PublicKey } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { findMarketLiquiditiesPda, LiquiditySource } from "../npm-client";
import { getProtocolProgram } from "./util";
export async function processMarketLiquidities() {
  const marketPk = new PublicKey(process.argv[3]);
  console.log(marketPk.toBase58());

  const forOutcome = process.argv[4] === "true";
  const outcome = parseInt(process.argv[5]);
  const price = parseFloat(process.argv[6]);

  const program = await getProtocolProgram();

  const marketLiquiditiesPk = (
    await findMarketLiquiditiesPda(program, marketPk)
  ).data.pda;

  await runInstruction(program, marketPk, marketLiquiditiesPk, forOutcome, [
    { outcome, price },
  ]);
}

async function runInstruction(
  program: Program,
  marketPk: PublicKey,
  marketLiquiditiesPk: PublicKey,
  sourceForOutcome: boolean,
  sourceLiquidities: LiquiditySource[],
) {
  return await program.methods
    .updateMarketLiquiditiesWithCrossLiquidity(
      sourceForOutcome,
      sourceLiquidities,
    )
    .accounts({
      market: marketPk,
      marketLiquidities: marketLiquiditiesPk,
    })
    .rpc();
}
