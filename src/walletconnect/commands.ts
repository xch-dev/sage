import { z } from 'zod';

const safeAmount = z.number().or(z.string());

const coinType = z.object({
  parent_coin_info: z.string(),
  puzzle_hash: z.string(),
  amount: safeAmount,
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

const assetAmount = z.object({
  assetId: z.string(),
  hiddenPuzzleHash: z.string().optional(),
  amount: safeAmount,
});

const nft = z.object({
  launcherId: z.string(),
  collectionId: z.string().nullable(),
  collectionName: z.string().nullable(),
  minterDid: z.string().nullable(),
  ownerDid: z.string().nullable(),
  name: z.string().nullable(),
  createdHeight: z.number().nullable(),
  coinId: z.string(),
  address: z.string(),
  royaltyAddress: z.string(),
  royaltyTenThousandths: z.number(),
  dataUris: z.array(z.string()),
  dataHash: z.string().nullable(),
  metadataUris: z.array(z.string()),
  metadataHash: z.string().nullable(),
  licenseUris: z.array(z.string()),
  licenseHash: z.string().nullable(),
  editionNumber: z.number().nullable(),
  editionTotal: z.number().nullable(),
});

const nftMint = z.object({
  address: z.string().optional(),
  royaltyAddress: z.string().optional(),
  royaltyTenThousandths: z.number().optional(),
  dataUris: z.array(z.string()).optional(),
  dataHash: z.string().optional(),
  metadataUris: z.array(z.string()).optional(),
  metadataHash: z.string().optional(),
  licenseUris: z.array(z.string()).optional(),
  licenseHash: z.string().optional(),
  editionNumber: z.number().optional(),
  editionTotal: z.number().optional(),
});

const handleRecord = z.object({
  handle: z.string(),
  handleHash: z.string(),
  address: z.string(),
  status: z.enum(['verified', 'changed']),
  previousAddress: z.string().nullable(),
  lastSentAddress: z.string().nullable(),
  changedFrom: z.string().nullable(),
  cached: z.boolean(),
  verification: z.enum([
    'verified',
    'outdated',
    'unverified',
    'stale',
    'expired',
  ]),
  verificationReason: z.string().nullable(),
  ownerLauncherId: z.string(),
  resolvedLauncherId: z.string(),
  expiration: z.number(),
  slotCoinId: z.string(),
  slotConfirmationHeight: z.number(),
  resolvedCoinId: z.string(),
  resolvedConfirmationHeight: z.number(),
  updatedHeight: z.number(),
  verifiedHeight: z.number(),
  verifiedTimestamp: z.number(),
  lastVerifiedAt: z.number(),
});

const handleSlot = z.object({
  registryLauncherId: z.string(),
  counter: z.number(),
  leftHandleHash: z.string(),
  rightHandleHash: z.string(),
  parentId: z.string(),
  parentParentId: z.string(),
  parentInnerPuzzleHash: z.string(),
  indexedHeight: z.number(),
});

const pendingTransfer = z.object({
  newOwnerLauncherId: z.string(),
  newResolvedLauncherId: z.string(),
  minimumExecutionHeight: z.number(),
  initiatorCoinId: z.string(),
  executorCoinId: z.string(),
  updateConfirmationHeight: z.number(),
  updateSlotCoinId: z.string().nullable(),
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
        hardened: z.boolean().optional(),
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
            amount: safeAmount.nullable(),
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
  chia_cancelOffer: {
    paramsType: z.object({
      id: z.string(),
      fee: safeAmount.optional(),
    }),
    returnType: z.object({}),
    confirm: true,
  },
  chia_getNfts: {
    paramsType: z.object({
      limit: z.number().optional(),
      offset: z.number().optional(),
      collectionId: z.string().optional(),
    }),
    returnType: z.object({
      nfts: z.array(nft),
    }),
    confirm: false,
  },
  chia_send: {
    paramsType: z.object({
      assetId: z.string().optional(),
      amount: safeAmount,
      fee: safeAmount.optional(),
      address: z.string(),
      memos: z.array(z.string()).optional(),
    }),
    returnType: z.object({}),
    confirm: true,
  },
  chia_bulkMintNfts: {
    paramsType: z.object({
      did: z.string(),
      nfts: z.array(nftMint),
      fee: safeAmount.optional(),
    }),
    returnType: z.object({
      nftIds: z.array(z.string()),
    }),
    confirm: true,
  },
  chia_getAddress: {
    paramsType: z.object({}),
    returnType: z.object({
      address: z.string(),
    }),
    confirm: false,
  },
  chia_signMessageByAddress: {
    paramsType: z.object({
      message: z.string(),
      address: z.string(),
    }),
    returnType: z.object({
      publicKey: z.string(),
      signature: z.string(),
    }),
    confirm: true,
  },
  // XCHandles reads. Every handle is verified against the blockchain, and the
  // result carries its verification state and any change warning, so that a site
  // never has to trust a bare address. XCHandles failures set the JSON-RPC
  // error's data to JSON with a `code`, which is a HandleErrorCode such as
  // 'expired' or 'unavailable', or 'rate_limited' when sites resolve too many
  // handles in a short time.
  //
  // There's no reverse lookup for sites: which handles this wallet has looked up
  // for other addresses is private.
  chia_resolveHandle: {
    paramsType: z.object({ handle: z.string() }),
    returnType: handleRecord,
    confirm: false,
  },
  chia_getHandle: {
    paramsType: z.object({ handle: z.string() }),
    returnType: z.object({
      handle: handleRecord,
      slot: handleSlot,
      pendingTransferStatus: z.enum([
        'none',
        'verified',
        'unverified',
        'unavailable',
      ]),
      pendingTransfer: pendingTransfer.nullable(),
    }),
    confirm: false,
  },
  // Verified handles whose owner NFT or DID is in this wallet. Other handles
  // Sage knows about aren't shared with sites.
  chia_getOwnedHandles: {
    paramsType: z
      .object({
        limit: z.number().optional(),
        offset: z.number().optional(),
      })
      .optional(),
    returnType: z.object({
      handles: z.array(handleRecord),
      total: z.number(),
    }),
    confirm: false,
  },
  // Update the display name, ticker, icon and description of a CAT this wallet
  // already holds (the same record update_cat edits). Nothing is signed; the user
  // reviews the suggestion against the asset id and approves or rejects it.
  chia_updateCat: {
    paramsType: z.object({
      assetId: z
        .string()
        .regex(
          /^(0x)?[0-9a-fA-F]{64}$/,
          'assetId must be a 32-byte hex asset id',
        ),
      name: z.string().trim().min(1).max(64).optional(),
      ticker: z.string().trim().min(1).max(16).optional(),
      iconUrl: z
        .string()
        .url()
        .max(512)
        .refine((url) => url.startsWith('https://'), 'iconUrl must be https')
        .optional(),
      description: z.string().trim().max(280).optional(),
    }),
    returnType: z.object({}),
    confirm: true,
  },
} as const;

// Define a union of valid commands
export type WalletConnectCommand = keyof typeof walletConnectCommands;

type Config<C extends WalletConnectCommand> = (typeof walletConnectCommands)[C];

export type Params<C extends WalletConnectCommand> = z.infer<
  Config<C>['paramsType']
>;

export type Return<C extends WalletConnectCommand> = z.infer<
  Config<C>['returnType']
>;

// Function to parse params based on the command
export const parseCommand = <C extends WalletConnectCommand>(
  command: C,
  params: unknown,
): z.infer<Config<C>['paramsType']> => {
  const commandInfo = walletConnectCommands[command];
  return commandInfo.paramsType.parse(params) as Params<C>;
};
