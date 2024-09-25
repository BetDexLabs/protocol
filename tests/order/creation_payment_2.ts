import assert from "assert";
import { createWalletWithBalance } from "../util/test_util";
import { monaco } from "../util/wrappers";

/*
 * Order Creation Payment 2
 *
 * This test case covers situation when patron creates only for orders for different outcomes and recieves refund while doing so.
 *
 * Scenario 1:
 *
 * Patron creates an order of X @ 2.00 for each outcome of the market with three outcomes.
 * Patron's starting market position is [0, 0, 0] and final market position should be [-X, -X, -X].
 * First two orders should each take payment of X, while third should refund X. Total payment taken should be X.
 *
 * Scenario 2:
 *
 * Patron creates an order of X @ 3.00 for each outcome of the market with three outcomes.
 * Patron's starting market position is [0, 0, 0] and final market position should be [0, 0, 0].
 * First two orders should each take payment of X, while third should refund 2*X. Total payment taken should be 0.
 *
 * Scenario 3:
 *
 * Patron creates an order of X @ 4.00 for each outcome of the market with three outcomes.
 * Patron's starting market position is [0, 0, 0] and final market position should be [X, X, X].
 * First two orders should each take payment of X, while third should refund 2*X. Total payment taken should be 0.
 *
 */
describe("Order Creation Payment 2", () => {
  it("Scenario 1: for all outcomes 10.00 @ 2.00", async () => {
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

    // Create For 10 for Outcome A
    await market.forOrder(outcomeA, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [0, 10, 10] },
        {
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 2, sources: [] },
          ],
        },
        10,
        90,
      ],
    );

    // Create For 10 for Outcome B
    await market.forOrder(outcomeB, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [10, 10, 20] },
        {
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 2, sources: [] },
            { liquidity: 10, outcome: 1, price: 2, sources: [] },
          ],
        },
        20,
        80,
      ],
    );

    // Create For 10 for Outcome C
    await market.forOrder(outcomeC, 10.0, price, purchaser);

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
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 2, sources: [] },
            { liquidity: 10, outcome: 1, price: 2, sources: [] },
            { liquidity: 10, outcome: 2, price: 2, sources: [] },
          ],
        },
        20,
        80,
      ],
    );
  });

  it("Scenario 2: for all outcomes 10.00 @ 3.00", async () => {
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

    // Create For 10 for Outcome A
    await market.forOrder(outcomeA, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [0, 10, 10] },
        {
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 3, sources: [] },
          ],
        },
        10,
        90,
      ],
    );

    // Create For 10 for Outcome B
    await market.forOrder(outcomeB, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [10, 10, 20] },
        {
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 3, sources: [] },
            { liquidity: 10, outcome: 1, price: 3, sources: [] },
          ],
        },
        20,
        80,
      ],
    );

    // Create For 10 for Outcome C
    await market.forOrder(outcomeC, 10.0, price, purchaser);

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
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 3, sources: [] },
            { liquidity: 10, outcome: 1, price: 3, sources: [] },
            { liquidity: 10, outcome: 2, price: 3, sources: [] },
          ],
        },
        20,
        80,
      ],
    );
  });

  it("Scenario 3: for all outcomes 10.00 @ 4.00", async () => {
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

    // Create For 10 for Outcome A
    await market.forOrder(outcomeA, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [0, 10, 10] },
        {
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 4, sources: [] },
          ],
        },
        10,
        90,
      ],
    );

    // Create For 10 for Outcome B
    await market.forOrder(outcomeB, 10.0, price, purchaser);

    assert.deepEqual(
      await Promise.all([
        market.getMarketPosition(purchaser),
        market.getMarketLiquidities(),
        market.getEscrowBalance(),
        market.getTokenBalance(purchaser),
      ]),
      [
        { matched: [0, 0, 0], unmatched: [10, 10, 20] },
        {
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 4, sources: [] },
            { liquidity: 10, outcome: 1, price: 4, sources: [] },
          ],
        },
        20,
        80,
      ],
    );

    // Create For 10 for Outcome C
    await market.forOrder(outcomeC, 10.0, price, purchaser);

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
          liquiditiesAgainst: [],
          liquiditiesFor: [
            { liquidity: 10, outcome: 0, price: 4, sources: [] },
            { liquidity: 10, outcome: 1, price: 4, sources: [] },
            { liquidity: 10, outcome: 2, price: 4, sources: [] },
          ],
        },
        20,
        80,
      ],
    );
  });
});
