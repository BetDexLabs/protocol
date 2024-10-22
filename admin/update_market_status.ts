import {
  findAuthorisedOperatorsAccountPda,
  openMarket as openMarketClient,
  Operator,
  publishMarket,
  setMarketReadyToClose as setMarketReadyToCloseClient,
  settleMarket,
  suspendMarket,
  unpublishMarket,
  unsuspendMarket,
  updateMarketLocktimeToNow,
  voidMarket as setMarketReadyToVoidClient,
} from "../npm-admin-client/src";
import { checkResponse, getProtocolProgram } from "./util";
import { PublicKey } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import {
  findEscrowPda,
  findMarketMatchingQueuePda,
  findMarketOrderRequestQueuePda,
  MarketAccount,
  signAndSendInstructions,
} from "../npm-client";

// yarn run settleMarket <MARKET_ID> <WINNING_OUTCOME_INDEX>
// or tsc; ANCHOR_WALLET=~/.config/solana/id.json yarn ts-node client.ts settle_market <MARKET_ID> <WINNING_OUTCOME_INDEX>

export async function openMarket() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run openMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const marketPk = new PublicKey(marketID);

  const protocolProgram = await getProtocolProgram();
  checkResponse(await openMarketClient(protocolProgram, marketPk, false));
}

export async function settle_market() {
  if (process.argv.length != 5) {
    console.log(
      "Usage: yarn run settleMarket <MARKET_ID> <WINNING_OUTCOME_INDEX>",
    );
    process.exit(1);
  }

  const marketID = process.argv[3];
  const winningOutcomeIndex = parseInt(process.argv[4], 10);
  const marketPk = new PublicKey(marketID);

  const protocolProgram = await getProtocolProgram();

  checkResponse(
    await settleMarket(protocolProgram, marketPk, winningOutcomeIndex),
  );
}

export async function voidMarket() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run voidMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const marketPk = new PublicKey(marketID);

  const protocolProgram = await getProtocolProgram();
  checkResponse(await setMarketReadyToVoidClient(protocolProgram, marketPk));
}

export async function setMarketReadyToClose() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run setMarketReadyToClose <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const marketPk = new PublicKey(marketID);

  const protocolProgram = await getProtocolProgram();
  checkResponse(await setMarketReadyToCloseClient(protocolProgram, marketPk));
}

export async function publish_market() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run publishMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const marketPk = new PublicKey(marketID);

  const protocolProgram = await getProtocolProgram();
  checkResponse(await publishMarket(protocolProgram, marketPk));
}

export async function unpublish_market() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run unpublishMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const marketPk = new PublicKey(marketID);

  const protocolProgram = await getProtocolProgram();
  checkResponse(await unpublishMarket(protocolProgram, marketPk));
}

export async function suspend_market() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run suspendMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const marketPk = new PublicKey(marketID);

  const protocolProgram = await getProtocolProgram();
  checkResponse(await suspendMarket(protocolProgram, marketPk));
}

export async function unsuspend_market() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run unsuspendMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const marketPk = new PublicKey(marketID);

  const protocolProgram = await getProtocolProgram();
  checkResponse(await unsuspendMarket(protocolProgram, marketPk));
}

export async function lockMarket() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run lockMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const marketPk = new PublicKey(marketID);

  const protocolProgram = (await getProtocolProgram()) as Program;
  checkResponse(await updateMarketLocktimeToNow(protocolProgram, marketPk));
}

export async function forceVoidMarket() {
  if (process.argv.length != 4) {
    console.log("Usage: yarn run forceVoidMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketID = process.argv[3];

  const marketPk = new PublicKey(marketID);
  const program = await getProtocolProgram();

  const [marketEscrow, market] = await Promise.all([
    findEscrowPda(program, marketPk),
    (await program.account.market.fetch(marketPk)) as MarketAccount,
  ]);

  const authorisedOperators = await findAuthorisedOperatorsAccountPda(
    program,
    Operator.MARKET,
  );
  const marketMatchingQueuePk = market.marketStatus.initializing
    ? null
    : (await findMarketMatchingQueuePda(program, marketPk)).data.pda;
  const orderRequestQueuePk = market.marketStatus.initializing
    ? null
    : (await findMarketOrderRequestQueuePda(program, marketPk)).data.pda;

  const instruction = await program.methods
    .forceVoidMarket()
    .accounts({
      market: new PublicKey(marketPk),
      marketEscrow: marketEscrow.data.pda,
      authorisedOperators: authorisedOperators.data.pda,
      marketOperator: program.provider.publicKey,
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      marketMatchingQueue: marketMatchingQueuePk,
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore
      orderRequestQueue: orderRequestQueuePk,
    })
    .instruction();

  const transaction = await signAndSendInstructions(program, [instruction]);
  console.log(`Transaction: ${transaction.data.signature}`);
}

export async function forceUnsettledCount() {
  if (process.argv.length != 5) {
    console.log("Usage: yarn run forceUnsettledCount <MARKET_ID> <NEW_COUNT>");
    process.exit(1);
  }

  const marketID = process.argv[3];
  const newCount = process.argv[4];

  const program = await getProtocolProgram();
  const marketPk = new PublicKey(marketID);

  const marketEscrow = await findEscrowPda(program, marketPk);
  const authorisedOperators = await findAuthorisedOperatorsAccountPda(
    program,
    Operator.MARKET,
  );

  const instruction = await program.methods
    .forceUnsettledCount(newCount)
    .accounts({
      market: marketPk,
      marketEscrow: marketEscrow.data.pda,
      marketOperator: program.provider.publicKey,
      authorisedOperators: authorisedOperators.data.pda,
    })
    .instruction();

  const transaction = await signAndSendInstructions(program, [instruction]);
  console.log(`Transaction: ${transaction.data.signature}`);
}
