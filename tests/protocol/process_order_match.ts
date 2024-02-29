import assert from "assert";
import { monaco } from "../util/wrappers";
import { createWalletWithBalance } from "../util/test_util";

describe("Matching Crank", () => {
  it("Success", async () => {
    // GIVEN

    // Create market, purchaser
    const [purchaserA, purchaserB, market] = await Promise.all([
      createWalletWithBalance(monaco.provider),
      createWalletWithBalance(monaco.provider),
      monaco.create3WayMarket([3.0]),
    ]);
    await market.airdrop(purchaserA, 100.0);
    await market.airdrop(purchaserB, 100.0);

    const againstPk = await market.againstOrder(1, 10, 3.0, purchaserA);
    const forPk = await market.forOrder(1, 10, 3.0, purchaserB);

    await market.processMatchingQueue();

    assert.deepEqual(
      await Promise.all([monaco.getOrder(againstPk), monaco.getOrder(forPk)]),
      [
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
      ],
    );
  });

  /**
   * Testing what happens when limit is reach for partial matches generated on creation.
   */
  it("Success: but not all liquidity matched", async () => {
    // GIVEN

    // Create market, purchaser
    const [purchaserA, purchaserB, market] = await Promise.all([
      createWalletWithBalance(monaco.provider),
      createWalletWithBalance(monaco.provider),
      monaco.create3WayMarket([
        3.0, 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7, 3.8, 3.9, 4.0,
      ]),
    ]);
    await market.airdrop(purchaserA, 1000.0);
    await market.airdrop(purchaserB, 1000.0);

    const against01Pk = await market.againstOrder(1, 10, 3.0, purchaserA);
    const against02Pk = await market.againstOrder(1, 10, 3.1, purchaserA);
    const against03Pk = await market.againstOrder(1, 10, 3.2, purchaserA);
    const against04Pk = await market.againstOrder(1, 10, 3.3, purchaserA);
    const against05Pk = await market.againstOrder(1, 10, 3.4, purchaserA);
    const against06Pk = await market.againstOrder(1, 10, 3.5, purchaserA);
    const against07Pk = await market.againstOrder(1, 10, 3.6, purchaserA);
    const against08Pk = await market.againstOrder(1, 10, 3.7, purchaserA);
    const against09Pk = await market.againstOrder(1, 10, 3.8, purchaserA);
    const against10Pk = await market.againstOrder(1, 10, 3.9, purchaserA);
    const against11Pk = await market.againstOrder(1, 10, 4.0, purchaserA);
    const forPk = await market.forOrder(1, 110, 3.0, purchaserB);

    assert.equal(await market.getMarketMatchingQueueLength(), 16);

    await market.processMatchingQueue();

    assert.deepEqual(
      await Promise.all([
        monaco.getOrder(against01Pk),
        monaco.getOrder(against02Pk),
        monaco.getOrder(against03Pk),
        monaco.getOrder(against04Pk),
        monaco.getOrder(against05Pk),
        monaco.getOrder(against06Pk),
        monaco.getOrder(against07Pk),
        monaco.getOrder(against08Pk),
        monaco.getOrder(against09Pk),
        monaco.getOrder(against10Pk),
        monaco.getOrder(against11Pk),
        monaco.getOrder(forPk),
        market.getMarketMatchingQueueLength(),
      ]),
      [
        { stakeUnmatched: 10, stakeVoided: 0, status: { open: {} } },
        { stakeUnmatched: 10, stakeVoided: 0, status: { open: {} } },
        { stakeUnmatched: 10, stakeVoided: 0, status: { open: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 0, stakeVoided: 0, status: { matched: {} } },
        { stakeUnmatched: 30, stakeVoided: 0, status: { matched: {} } },
        0,
      ],
    );
  });
});
