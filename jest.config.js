const jestConfig = {
    verbose: true,
    testMatch: ["**/tests/market/force_void_market.ts?(x)"],
    testPathIgnorePatterns: [
        "tests/setup.ts",
        "tests/util/pdas.ts",
        "tests/util/test_util.ts",
        "tests/util/wrappers.ts",
        "tests/anchor/*",
        ],
    globalSetup: "<rootDir>/tests/setup.ts",
    setupFilesAfterEnv: ["<rootDir>/tests/util/test_util.ts"],
    testTimeout: 1200000,
};

module.exports = jestConfig;
