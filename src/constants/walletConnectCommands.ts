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

enum MempoolInclusionStatus {
  SUCCESS = 1, // Transaction added to mempool
  PENDING = 2, // Transaction not yet added to mempool
  FAILED = 3, // Transaction was invalid and dropped
}

// Convert the array into an object keyed by the `command`
export const walletConnectCommands = {
  chip0002_getPublicKeys: {
    requiresConfirmation: false,
    paramsType: z
      .object({
        limit: z.number().optional(),
        offset: z.number().optional(),
      })
      .optional(),
    returnType: z.array(z.string()),
  },
  chip0002_filterUnlockedCoins: {
    requiresConfirmation: false,
    paramsType: z.object({ coinNames: z.array(z.string()).min(1) }),
    returnType: z.array(z.string()),
  },
  chip0002_getAssetCoins: {
    requiresConfirmation: false,
    paramsType: z.object({
      type: z.string().nullable(),
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
  },
  chip0002_getAssetBalance: {
    requiresConfirmation: false,
    paramsType: z.object({
      type: z.string().nullable(),
      assetId: z.string().nullable(),
    }),
    returnType: z.object({
      confirmed: z.string(),
      spendable: z.string(),
      spendableCoinCount: z.number(),
    }),
  },
  chip0002_signCoinSpends: {
    requiresConfirmation: true,
    paramsType: z.object({
      coinSpends: z.array(coinType),
      partialSign: z.boolean().optional(),
    }),
    returnType: z.string(),
  },
  chip0002_signMessage: {
    requiresConfirmation: true,
    paramsType: z.object({
      message: z.string(),
      publicKey: z.string(),
    }),
    returnType: z.string(),
  },
  chip0002_sendTransaction: {
    requiresConfirmation: false,
    paramsType: z.object({ spendBundle: spendBundleType }),
    returnType: z.object({
      status: z
        .number()
        .refine((val) => Object.values(MempoolInclusionStatus).includes(val)),
      error: z.string().nullable(),
    }),
  },
} as const;

// Define a union of valid commands
export type WalletConnectCommand = keyof typeof walletConnectCommands;

type CommandConfig<C extends WalletConnectCommand> =
  (typeof walletConnectCommands)[C];

// Function to parse params based on the command
export const parseCommand = <C extends WalletConnectCommand>(
  command: C,
  params: unknown,
): z.infer<CommandConfig<C>['paramsType']> => {
  const commandInfo = walletConnectCommands[command];
  return commandInfo.paramsType.parse(params);
};
