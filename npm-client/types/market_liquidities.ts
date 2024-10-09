import { PublicKey } from "@solana/web3.js";
import { BN } from "@coral-xyz/anchor";
import { GetAccount } from "./get_account";

export type MarketLiquiditiesAccounts = {
  accounts: GetAccount<MarketLiquidities>[];
};

export type MarketLiquidities = {
  market: PublicKey;
  enableCrossMatching: boolean;
  stakeTraded: BN;
  volumeTraded: BN;
  liquiditiesFor: MarketLiquidity[];
  liquiditiesAgainst: MarketLiquidity[];
};

export type MarketLiquidity = {
  outcome: number;
  price: number;
  sources: LiquiditySource[];
  liquidity: BN;
};

export type LiquiditySource = {
  outcome: number;
  price: number;
};
