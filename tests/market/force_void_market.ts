import { createWalletWithBalance } from "../util/test_util";
import { monaco } from "../util/wrappers";
import assert from "assert";

describe("Force void market", () => {
  it("void while items remain in matching queue", async () => {
    const price = 2.0;
    const [p1, p2, p3, p4, p5, market] = await Promise.all([
      createWalletWithBalance(monaco.provider),
      createWalletWithBalance(monaco.provider),
      createWalletWithBalance(monaco.provider),
      createWalletWithBalance(monaco.provider),
      createWalletWithBalance(monaco.provider),
      monaco.create3WayMarket(
        [price],
        false,
        0,
        undefined,
        undefined,
        undefined,
        { cancelUnmatched: {} },
      ),
    ]);

    // set up purchasers
    await market.airdrop(p1, 100.0);
    await market.airdrop(p2, 200.0);
    await market.airdrop(p3, 300.0);
    await market.airdrop(p4, 400.0);
    await market.airdrop(p5, 400.0);
    const p1Balance = await market.getTokenBalance(p1.publicKey);
    const p2Balance = await market.getTokenBalance(p2.publicKey);
    const p3Balance = await market.getTokenBalance(p3.publicKey);
    const p4Balance = await market.getTokenBalance(p4.publicKey);
    const p5Balance = await market.getTokenBalance(p4.publicKey);

    // create orders
    const p1OrderPk = await market.forOrder(0, 10.0, price, p1);
    const p2OrderPk = await market.againstOrder(0, 20.0, price, p2);
    const p3OrderPk = await market.forOrder(0, 20.0, price, p3);
    const p4OrderPk = await market.forOrder(0, 10.0, price, p4);

    await market.processMatchingQueue();
    let matchingQueue = await market.getMarketMatchingQueueLength();
    assert.equal(matchingQueue, 0);

    await market.updateMarketLockTimeToNow();

    await market.cancelOrderPostMarketLock(p3OrderPk);
    await market.cancelOrderPostMarketLock(p4OrderPk);

    const p3Order = await monaco.getOrder(p3OrderPk);
    const p4Order = await monaco.getOrder(p4OrderPk);
    assert.deepEqual({ matched: {} }, p3Order.status);
    assert.deepEqual({ cancelled: {} }, p4Order.status);

    await market.updateMarketLockTime(Date.now() / 1000 + 10);

    const liquidity = await market.getMarketLiquidities();
    console.log(liquidity);

    matchingQueue = await market.getMarketMatchingQueueLength();
    assert.equal(matchingQueue, 0);

    console.log("before 5");
    console.log(await monaco.getOrder(p3OrderPk));
    console.log(await monaco.getOrder(p4OrderPk));

    const p5OrderPk = await market.againstOrder(0, 20.0, price, p5);
    matchingQueue = await market.getMarketMatchingQueueLength();
    assert.equal(matchingQueue, 2);

    console.log("after 5");
    console.log(await monaco.getOrder(p3OrderPk));
    console.log(await monaco.getOrder(p4OrderPk));

    try {
      await market.processMatchingQueue();
      fail("expected exception");
    } catch (e) {
      console.log(e);
    }
    matchingQueue = await market.getMarketMatchingQueueLength();
    assert.equal(matchingQueue, 2);

    console.log("before void");
    console.log(await monaco.getOrder(p4OrderPk));

    // force void market
    await market.voidMarket(true);

    console.log("after void");
    console.log(await monaco.getOrder(p4OrderPk));

    // void market positions to return funds to purchasers
    await market.voidMarketPositionForPurchaser(p1.publicKey);
    await market.voidMarketPositionForPurchaser(p2.publicKey);
    await market.voidMarketPositionForPurchaser(p3.publicKey);
    await market.voidMarketPositionForPurchaser(p4.publicKey);
    await market.voidMarketPositionForPurchaser(p5.publicKey);

    // check balances
    const p1BalanceAfter = await market.getTokenBalance(p1.publicKey);
    const p2BalanceAfter = await market.getTokenBalance(p2.publicKey);
    const p3BalanceAfter = await market.getTokenBalance(p3.publicKey);
    const p4BalanceAfter = await market.getTokenBalance(p4.publicKey);
    const p5BalanceAfter = await market.getTokenBalance(p5.publicKey);
    assert.equal(p1Balance, p1BalanceAfter);
    assert.equal(p2Balance, p2BalanceAfter);
    assert.equal(p3Balance, p3BalanceAfter);
    assert.equal(p4Balance, p4BalanceAfter);
    assert.equal(p5Balance, p5BalanceAfter);

    console.log((await market.getAccount()).unsettledAccountsCount);
    console.log(await monaco.getOrder(p1OrderPk));
    console.log(await monaco.getOrder(p2OrderPk));
    console.log(await monaco.getOrder(p3OrderPk));
    console.log(await monaco.getOrder(p4OrderPk));
    console.log(await monaco.getOrder(p5OrderPk));

    // ensure market voiding can be completed
    await market.voidOrder(p1OrderPk);
    console.log((await market.getAccount()).unsettledAccountsCount);
    await market.voidOrder(p2OrderPk);
    console.log((await market.getAccount()).unsettledAccountsCount);
    await market.voidOrder(p3OrderPk);
    console.log((await market.getAccount()).unsettledAccountsCount);
    await market.voidOrder(p4OrderPk);
    console.log((await market.getAccount()).unsettledAccountsCount);
    await market.voidOrder(p5OrderPk);
    console.log((await market.getAccount()).unsettledAccountsCount);
    await market.completeVoid();
    const voidedMarket = await monaco.program.account.market.fetch(market.pk);
    assert.ok(voidedMarket.marketStatus.voided);

    // set market ready to close
    await market.readyToClose();
    const closingMarket = await monaco.program.account.market.fetch(market.pk);
    assert.ok(closingMarket.marketStatus.readyToClose);

    // ensure market can be closed
    await market.closeOrder(p1OrderPk);
    await market.closeOrder(p2OrderPk);
    await market.closeOrder(p3OrderPk);
    await market.closeOrder(p4OrderPk);
    await market.closeOrder(p5OrderPk);
    await market.closeMarketPosition(p1.publicKey);
    await market.closeMarketPosition(p2.publicKey);
    await market.closeMarketPosition(p3.publicKey);
    await market.closeMarketPosition(p4.publicKey);
    await market.closeMarketPosition(p5.publicKey);
    await market.closeMarketMatchingPool(0, price, true);
    await market.closeMarketMatchingPool(0, price, false);
    await market.closeOutcome(0);
    await market.closeOutcome(1);
    await market.closeOutcome(2);
    await market.closeMarketQueues();
    await market.close();

    try {
      await monaco.program.account.market.fetch(market.pk);
      assert.fail("expected Account does not exist or has no data...");
    } catch (e) {
      assert.equal(
        e.message,
        `Account does not exist or has no data ${market.pk.toBase58()}`,
      );
    }
  });
});
