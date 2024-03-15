import assert from "assert";
import { createWalletWithBalance } from "../util/test_util";
import { monaco } from "../util/wrappers";

/*
 */
describe("Order Creation Cross Liquidity", () => {
  it("Scenario 1: 3-way market generated from for liquidity", async () => {
    // Given
    const prices = [2.1, 3.0, 5.25];

    // Create market, purchaser
    const [purchaser, market] = await Promise.all([
      createWalletWithBalance(monaco.provider),
      monaco.create3WayMarket(prices),
    ]);
    await market.airdrop(purchaser, 1000.0);

    await market.forOrder(0, 100.0, prices[0], purchaser);
    await market.forOrder(1, 100.0, prices[1], purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [200, 800],
    );

    await market.updateMarketLiquiditiesWithCrossLiquidity(
      true,
      [
        { outcome: 0, price: prices[0] },
        { outcome: 1, price: prices[1] },
      ],
      { outcome: 2, price: prices[2] },
    );

    assert.deepEqual(await market.getMarketLiquidities(), {
      liquiditiesAgainst: [
        {
          liquidity: 40,
          outcome: 2,
          price: 5.25,
          sources: [
            { outcome: 0, price: 2.1 },
            { outcome: 1, price: 3 },
          ],
        },
      ],
      liquiditiesFor: [
        { liquidity: 100, outcome: 0, price: 2.1, sources: [] },
        { liquidity: 100, outcome: 1, price: 3, sources: [] },
      ],
    });
  });
});
