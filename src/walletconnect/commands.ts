import { z } from 'zod';

const coinType = z.object({
  parent_coin_info: z.string(),
  puzzle_hash: z.string(),
  amount: z.number(),
});

const coinSpendType = z.object({
  coin: coinType,
  puzzle_reveal: z.string(),
  solution: z.string(),
});

const spendBundleType = z.object({
  coin_spends: z.array(coinSpendType),
  aggregated_signature: z.string(),
});

const safeAmount = z.number().or(z.string());

const assetAmount = z.object({
  assetId: z.string(),
  amount: safeAmount,
});

enum MempoolInclusionStatus {
  SUCCESS = 1, // Transaction added to mempool
  PENDING = 2, // Transaction not yet added to mempool
  FAILED = 3, // Transaction was invalid and dropped
}

// Convert the array into an object keyed by the `command`
export const walletConnectCommands = {
  chip0002_chainId: {
    paramsType: z.object({}).optional(),
    returnType: z.string(),
    confirm: false,
  },
  chip0002_connect: {
    paramsType: z
      .object({
        eager: z.boolean().optional(),
      })
      .optional(),
    returnType: z.boolean(),
    confirm: false,
  },
  chip0002_getPublicKeys: {
    paramsType: z
      .object({
        limit: z.number().optional(),
        offset: z.number().optional(),
      })
      .optional(),
    returnType: z.array(z.string()),
    confirm: false,
  },
  chip0002_filterUnlockedCoins: {
    paramsType: z.object({ coinNames: z.array(z.string()).min(1) }),
    returnType: z.array(z.string()),
    confirm: false,
  },
  chip0002_getAssetCoins: {
    paramsType: z.object({
      type: z.enum(['cat', 'nft', 'did']).nullable(),
      assetId: z.string().nullable(),
      includedLocked: z.boolean().optional(),
      offset: z.number().optional(),
      limit: z.number().optional(),
    }),
    returnType: z.array(
      z.object({
        coin: coinType,
        coinName: z.string(),
        puzzle: z.string(),
        confirmedBlockIndex: z.number(),
        locked: z.boolean(),
        lineageProof: z
          .object({
            parentName: z.string().nullable(),
            innerPuzzleHash: z.string().nullable(),
            amount: z.number().nullable(),
          })
          .nullable(),
      }),
    ),
    confirm: false,
  },
  chip0002_getAssetBalance: {
    paramsType: z.object({
      type: z.enum(['cat', 'nft', 'did']).nullable(),
      assetId: z.string().nullable(),
    }),
    returnType: z.object({
      confirmed: z.string(),
      spendable: z.string(),
      spendableCoinCount: z.number(),
    }),
    confirm: false,
  },
  chip0002_signCoinSpends: {
    paramsType: z.object({
      coinSpends: z.array(coinSpendType),
      partialSign: z.boolean().optional(),
    }),
    returnType: z.string(),
    confirm: true,
  },
  chip0002_signMessage: {
    paramsType: z.object({
      message: z.string(),
      publicKey: z.string(),
    }),
    returnType: z.string(),
    confirm: true,
  },
  chip0002_sendTransaction: {
    paramsType: z.object({ spendBundle: spendBundleType }),
    returnType: z.object({
      status: z
        .number()
        .refine((val) => Object.values(MempoolInclusionStatus).includes(val)),
      error: z.string().nullable(),
    }),
    confirm: false,
  },
  chia_transfer: {
    paramsType: z.object({
      to: z.string(),
      amount: safeAmount,
      memos: z.array(z.string()).optional(),
      assetId: z.string(),
    }),
    returnType: z.object({
      id: z.string(),
    }),
    confirm: true,
  },
  chia_takeOffer: {
    paramsType: z.object({
      offer: z.string(),
      fee: safeAmount.optional(),
    }),
    returnType: z.object({
      id: z.string(),
    }),
    confirm: true,
  },
  chia_createOffer: {
    paramsType: z.object({
      offerAssets: z.array(assetAmount),
      requestAssets: z.array(assetAmount),
      fee: safeAmount.optional(),
    }),
    returnType: z.object({
      id: z.string(),
      offer: z.string(),
    }),
    confirm: true,
  },
} as const;

// Define a union of valid commands
export type WalletConnectCommand = keyof typeof walletConnectCommands;

type Config<C extends WalletConnectCommand> = (typeof walletConnectCommands)[C];

export type Params<C extends WalletConnectCommand> = z.infer<
  Config<C>['paramsType']
>;

// Function to parse params based on the command
export const parseCommand = <C extends WalletConnectCommand>(
  command: C,
  params: unknown,
): z.infer<Config<C>['paramsType']> => {
  const commandInfo = walletConnectCommands[command];
  return commandInfo.paramsType.parse(params);
};
