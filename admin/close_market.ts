import { PublicKey } from "@solana/web3.js";
import {
  findEscrowPda,
  findMarketMatchingQueuePda,
  findMarketCommissionPaymentQueuePda,
  findAuthorisedOperatorsAccountPda,
  Operator,
  findMarketFundingPda,
  voidMarket,
} from "../npm-admin-client";
import { getAnchorProvider, getProtocolProgram } from "./util";
import {
  findMarketLiquiditiesPda,
  findMarketOrderRequestQueuePda,
  getMarket,
  MarketMatchingPools,
  MarketPositions,
  Orders,
  Trades,
} from "../npm-client";
import { Program } from "@coral-xyz/anchor";
import console from "console";

export async function closeMarket() {
  const protocolProgram = await getProtocolProgram();

  if (process.argv.length != 4) {
    console.log("Usage: yarn closeMarket <MARKET_ID>");
    process.exit(1);
  }

  const marketPk = new PublicKey(process.argv[3]);
  const authorityPk = getAnchorProvider().wallet.publicKey;
  console.log(`Closing market ${marketPk} using authority ${authorityPk}`);

  // check if market exists
  const market = await getMarket(protocolProgram, marketPk);
  if (!market.success) {
    console.error(`Market ${marketPk} does not exist`);
    return;
  }
  const marketEscrowPk = await findEscrowPda(protocolProgram, marketPk);
  const marketFundingPk = await findMarketFundingPda(protocolProgram, marketPk);
  const authorisedOperatorsPk = await findAuthorisedOperatorsAccountPda(
    protocolProgram as Program,
    Operator.MARKET,
  );

  // check market's status
  let marketStatus = market.data.account.marketStatus;
  console.log(
    `Market ${marketPk} actual status ${JSON.stringify(marketStatus)}`,
  );

  if (marketStatus.initializing || marketStatus.open) {
    const response = await voidMarket(protocolProgram, marketPk);
    if (!response.success) {
      console.error(JSON.stringify(response.errors, null, 2));
      return;
    }

    marketStatus = await getMarketStatus(protocolProgram, marketPk);
  }

  if (marketStatus.voided || marketStatus.settled) {
    console.error(
      `Market ${marketPk} is ${JSON.stringify(
        marketStatus,
      )} so setting as ready-to-close`,
    );
    await protocolProgram.methods
      .setMarketReadyToClose()
      .accounts({
        market: marketPk,
        marketEscrow: marketEscrowPk.data.pda,
        marketFunding: marketFundingPk.data.pda,
        marketOperator: authorityPk,
        authorisedOperators: authorisedOperatorsPk.data.pda,
      })
      .rpc()
      .catch((e) => {
        console.error(e);
        throw e;
      });

    marketStatus = await getMarketStatus(protocolProgram, marketPk);
  }

  if (!marketStatus.readyToClose) {
    console.error(`Closing market ${marketPk} incorrect status`);
    return;
  }

  // check unclosedAccountsCount
  const unsettledAccountsCount = market.data.account.unsettledAccountsCount;
  const unclosedAccountsCount = market.data.account.unclosedAccountsCount;
  console.log(
    `Closing market ${marketPk} unsettled: ${unsettledAccountsCount} unclosed: ${unclosedAccountsCount}`,
  );
  if (unclosedAccountsCount > 0) {
    const orders = await Orders.orderQuery(protocolProgram)
      .filterByMarket(marketPk)
      .fetch();
    console.log(
      `Closing market ${marketPk} orders: ${orders.data.orderAccounts.length}`,
    );
    for (const order of orders.data.orderAccounts) {
      console.log(`Closing market ${marketPk} orders: ${order.publicKey}`);
      await protocolProgram.methods
        .closeOrder()
        .accounts({
          market: marketPk,
          payer: order.account.payer,
          order: order.publicKey,
        })
        .rpc()
        .catch((e) => {
          console.error(e);
          throw e;
        });
    }

    // --------

    const trades = await Trades.tradeQuery(protocolProgram)
      .filterByMarket(marketPk)
      .fetch();
    console.log(
      `Closing market ${marketPk} trades: ${trades.data.tradeAccounts.length}`,
    );
    for (const trade of trades.data.tradeAccounts) {
      console.log(`Closing market ${marketPk} trade: ${trade.publicKey}`);
      await protocolProgram.methods
        .closeTrade()
        .accounts({
          market: marketPk,
          payer: trade.account.payer,
          trade: trade.publicKey,
        })
        .rpc()
        .catch((e) => {
          console.error(e);
          throw e;
        });
    }

    // --------

    const marketPositions = await MarketPositions.marketPositionQuery(
      protocolProgram,
    )
      .filterByMarket(marketPk)
      .fetch();
    console.log(
      `Closing market ${marketPk} marketPositions: ${marketPositions.data.marketPositionAccounts.length}`,
    );
    for (const marketPosition of marketPositions.data.marketPositionAccounts) {
      console.log(
        `Closing market ${marketPk} marketPosition: ${marketPosition.publicKey}`,
      );
      await protocolProgram.methods
        .closeMarketPosition()
        .accounts({
          market: marketPk,
          purchaser: marketPosition.account.payer,
          marketPosition: marketPosition.publicKey,
        })
        .rpc()
        .catch((e) => {
          console.error(e);
          throw e;
        });
    }

    // --------

    const matchingPools = await MarketMatchingPools.marketMatchingPoolQuery(
      protocolProgram,
    )
      .filterByMarket(marketPk)
      .fetch();
    console.log(
      `Closing market ${marketPk} matchingPools: ${matchingPools.data.marketMatchingPools.length}`,
    );

    for (const matchingPool of matchingPools.data.marketMatchingPools) {
      console.log(
        `Closing market ${marketPk} matchingPool: ${matchingPool.publicKey}`,
      );
      await protocolProgram.methods
        .closeMarketMatchingPool()
        .accounts({
          market: marketPk,
          payer: matchingPool.account.payer,
          marketMatchingPool: matchingPool.publicKey,
        })
        .rpc()
        .catch((e) => {
          console.error(e);
          throw e;
        });
    }
  }
  // check market's authority
  const marketAuthorityPk = market.data.account.authority;
  console.log(
    `Closing market ${marketPk} actual authority ${marketAuthorityPk}`,
  );
  if (!marketAuthorityPk.equals(authorityPk)) {
    console.error(`Closing market ${marketPk} incorrect authority`);
    return;
  }

  // close
  const liquiditiesPk = await findMarketLiquiditiesPda(
    protocolProgram,
    marketPk,
  );
  const matchingQueuePk = await findMarketMatchingQueuePda(
    protocolProgram,
    marketPk,
  );
  const commissionPaymentQueuePk = await findMarketCommissionPaymentQueuePda(
    protocolProgram,
    marketPk,
  );
  const orderRequestQueuePk = await findMarketOrderRequestQueuePda(
    protocolProgram,
    marketPk,
  );

  console.log(`Closing market ${marketPk} executing`);
  await protocolProgram.methods
    .closeMarketQueues()
    .accounts({
      market: marketPk,
      liquidities: liquiditiesPk.data.pda,
      matchingQueue: matchingQueuePk.data.pda,
      commissionPaymentQueue: commissionPaymentQueuePk.data.pda,
      orderRequestQueue: orderRequestQueuePk.data.pda,
      authority: authorityPk,
    })
    .rpc()
    .catch((e) => {
      console.error(e);
    });
  await protocolProgram.methods
    .closeMarket()
    .accounts({
      market: marketPk,
      authority: authorityPk,
      marketEscrow: marketEscrowPk.data.pda,
      marketFunding: marketFundingPk.data.pda,
    })
    .rpc()
    .catch((e) => {
      console.error(e);
    });
  console.log(`Closing market ${marketPk} done`);
}

async function getMarketStatus(protocolProgram: Program, marketPk: PublicKey) {
  const market = await getMarket(protocolProgram, marketPk);
  if (!market.success) {
    throw new Error(`Market ${marketPk} does not exist`);
  }
  const marketStatus = market.data.account.marketStatus;
  console.log(`Market ${marketPk} updated to ${JSON.stringify(marketStatus)}`);
  return marketStatus;
}
