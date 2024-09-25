import assert from "assert";
import { createWalletWithBalance } from "../util/test_util";
import { monaco } from "../util/wrappers";

/*
 * Order Creation Payment 3
 *
 * This test case covers situation when patron creates only against orders for different outcomes and recieves refund while doing so.
 *
 * Scenario 1:
 *
 * Patron creates an order of X @ 2.00 against each outcome of the market with three outcomes.
 * Patron's starting market position is [0, 0, 0] and final market position should be [X, X, X].
 * First order should take payment of X, second should refund X and third should do nothing. Total payment taken should be 0.
 *
 * Scenario 2:
 *
 * Patron creates an order of X @ 3.00 against each outcome of the market with three outcomes.
 * Patron's starting market position is [0, 0, 0] and final market position should be [0, 0, 0].
 * First order should take payment of 2*X, Second and third should refund X each. Total payment taken should be 0.
 *
 * Scenario 3:
 *
 * Patron creates an order of X @ 4.00 against each outcome of the market with three outcomes.
 * Patron's starting market position is [0, 0, 0] and final market position should be [-X, -X, -X].
 * First order should take payment of 3*X, Second and third should refund X each. Total payment taken should be X.
 *
 */
describe("Order Creation Payment 3", () => {
  it("Scenario 1: against all outcomes 10.00 @ 2.00", async () => {
    // Given
    const outcomeA = 0;
    const outcomeB = 1;
    const outcomeC = 2;
    const price = 2.0;

    // Create market, purchaser
    const [purchaser, market] = await Promise.all([
      createWalletWithBalance(monaco.provider),
      monaco.create3WayMarket([price]),
    ]);
    await market.airdrop(purchaser, 100.0);

    // Create Against 10 for Outcome A
    await market.againstOrder(outcomeA, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [10, 0, 0] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 0, price: 2, sources: [] },
          ],
          liquiditiesFor: [],
        },
        10,
        90,
      ],
    );

    // Create Against 10 for Outcome B
    await market.againstOrder(outcomeB, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [10, 10, 0] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 1, price: 2, sources: [] },
            { liquidity: 10, outcome: 0, price: 2, sources: [] },
          ],
          liquiditiesFor: [],
        },
        10,
        90,
      ],
    );

    // Create Against 10 for Outcome C
    await market.againstOrder(outcomeC, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [10, 10, 10] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 2, price: 2, sources: [] },
            { liquidity: 10, outcome: 1, price: 2, sources: [] },
            { liquidity: 10, outcome: 0, price: 2, sources: [] },
          ],
          liquiditiesFor: [],
        },
        10,
        90,
      ],
    );
  });

  it("Scenario 2: against all outcomes 10.00 @ 3.00", async () => {
    // Given
    const outcomeA = 0;
    const outcomeB = 1;
    const outcomeC = 2;
    const price = 3.0;

    // Create market, purchaser
    const [purchaser, market] = await Promise.all([
      createWalletWithBalance(monaco.provider),
      monaco.create3WayMarket([price]),
    ]);
    await market.airdrop(purchaser, 100.0);

    // Create Against 10 for Outcome A
    await market.againstOrder(outcomeA, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [20, 0, 0] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 0, price: 3, sources: [] },
          ],
          liquiditiesFor: [],
        },
        20,
        80,
      ],
    );

    // Create Against 10 for Outcome B
    await market.againstOrder(outcomeB, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [20, 20, 0] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 1, price: 3, sources: [] },
            { liquidity: 10, outcome: 0, price: 3, sources: [] },
          ],
          liquiditiesFor: [],
        },
        20,
        80,
      ],
    );

    // Create Against 10 for Outcome C
    await market.againstOrder(outcomeC, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [20, 20, 20] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 2, price: 3, sources: [] },
            { liquidity: 10, outcome: 1, price: 3, sources: [] },
            { liquidity: 10, outcome: 0, price: 3, sources: [] },
          ],
          liquiditiesFor: [],
        },
        20,
        80,
      ],
    );
  });

  it("Scenario 3: against all outcomes 10.00 @ 4.00", async () => {
    // Given
    const outcomeA = 0;
    const outcomeB = 1;
    const outcomeC = 2;
    const price = 4.0;

    // Create market, purchaser
    const [purchaser, market] = await Promise.all([
      createWalletWithBalance(monaco.provider),
      monaco.create3WayMarket([price]),
    ]);
    await market.airdrop(purchaser, 100.0);

    // Create Against 10 for Outcome A
    await market.againstOrder(outcomeA, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [30, 0, 0] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 0, price: 4, sources: [] },
          ],
          liquiditiesFor: [],
        },
        30,
        70,
      ],
    );

    // Create Against 10 for Outcome B
    await market.againstOrder(outcomeB, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [30, 30, 0] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 1, price: 4, sources: [] },
            { liquidity: 10, outcome: 0, price: 4, sources: [] },
          ],
          liquiditiesFor: [],
        },
        30,
        70,
      ],
    );

    // Create Against 10 for Outcome C
    await market.againstOrder(outcomeC, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [30, 30, 30] },
        {
          liquiditiesAgainst: [
            { liquidity: 10, outcome: 2, price: 4, sources: [] },
            { liquidity: 10, outcome: 1, price: 4, sources: [] },
            { liquidity: 10, outcome: 0, price: 4, sources: [] },
          ],
          liquiditiesFor: [],
        },
        30,
        70,
      ],
    );
  });
});
