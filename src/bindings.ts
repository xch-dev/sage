
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

/** user-defined commands **/


export const commands = {
async initialize() : Promise<null> {
    return await TAURI_INVOKE("initialize");
},
async login(req: Login) : Promise<LoginResponse> {
    return await TAURI_INVOKE("login", { req });
},
async logout(req: Logout) : Promise<LogoutResponse> {
    return await TAURI_INVOKE("logout", { req });
},
async resync(req: Resync) : Promise<ResyncResponse> {
    return await TAURI_INVOKE("resync", { req });
},
async generateMnemonic(req: GenerateMnemonic) : Promise<GenerateMnemonicResponse> {
    return await TAURI_INVOKE("generate_mnemonic", { req });
},
async importKey(req: ImportKey) : Promise<ImportKeyResponse> {
    return await TAURI_INVOKE("import_key", { req });
},
async deleteKey(req: DeleteKey) : Promise<DeleteKeyResponse> {
    return await TAURI_INVOKE("delete_key", { req });
},
async deleteDatabase(req: DeleteDatabase) : Promise<DeleteDatabaseResponse> {
    return await TAURI_INVOKE("delete_database", { req });
},
async renameKey(req: RenameKey) : Promise<RenameKeyResponse> {
    return await TAURI_INVOKE("rename_key", { req });
},
async getKeys(req: GetKeys) : Promise<GetKeysResponse> {
    return await TAURI_INVOKE("get_keys", { req });
},
async getKey(req: GetKey) : Promise<GetKeyResponse> {
    return await TAURI_INVOKE("get_key", { req });
},
async getSecretKey(req: GetSecretKey) : Promise<GetSecretKeyResponse> {
    return await TAURI_INVOKE("get_secret_key", { req });
},
async sendXch(req: SendXch) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("send_xch", { req });
},
async bulkSendXch(req: BulkSendXch) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("bulk_send_xch", { req });
},
async combine(req: Combine) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("combine", { req });
},
async split(req: Split) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("split", { req });
},
async autoCombineXch(req: AutoCombineXch) : Promise<AutoCombineXchResponse> {
    return await TAURI_INVOKE("auto_combine_xch", { req });
},
async sendCat(req: SendCat) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("send_cat", { req });
},
async bulkSendCat(req: BulkSendCat) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("bulk_send_cat", { req });
},
async autoCombineCat(req: AutoCombineCat) : Promise<AutoCombineCatResponse> {
    return await TAURI_INVOKE("auto_combine_cat", { req });
},
async issueCat(req: IssueCat) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("issue_cat", { req });
},
async createDid(req: CreateDid) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("create_did", { req });
},
async bulkMintNfts(req: BulkMintNfts) : Promise<BulkMintNftsResponse> {
    return await TAURI_INVOKE("bulk_mint_nfts", { req });
},
async transferNfts(req: TransferNfts) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("transfer_nfts", { req });
},
async transferDids(req: TransferDids) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("transfer_dids", { req });
},
async normalizeDids(req: NormalizeDids) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("normalize_dids", { req });
},
async addNftUri(req: AddNftUri) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("add_nft_uri", { req });
},
async assignNftsToDid(req: AssignNftsToDid) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("assign_nfts_to_did", { req });
},
async signCoinSpends(req: SignCoinSpends) : Promise<SignCoinSpendsResponse> {
    return await TAURI_INVOKE("sign_coin_spends", { req });
},
async viewCoinSpends(req: ViewCoinSpends) : Promise<ViewCoinSpendsResponse> {
    return await TAURI_INVOKE("view_coin_spends", { req });
},
async submitTransaction(req: SubmitTransaction) : Promise<SubmitTransactionResponse> {
    return await TAURI_INVOKE("submit_transaction", { req });
},
async getSyncStatus(req: GetSyncStatus) : Promise<GetSyncStatusResponse> {
    return await TAURI_INVOKE("get_sync_status", { req });
},
async getVersion(req: GetVersion) : Promise<GetVersionResponse> {
    return await TAURI_INVOKE("get_version", { req });
},
async getDatabaseStats(req: GetDatabaseStats) : Promise<GetDatabaseStatsResponse> {
    return await TAURI_INVOKE("get_database_stats", { req });
},
async performDatabaseMaintenance(req: PerformDatabaseMaintenance) : Promise<PerformDatabaseMaintenanceResponse> {
    return await TAURI_INVOKE("perform_database_maintenance", { req });
},
async checkAddress(req: CheckAddress) : Promise<CheckAddressResponse> {
    return await TAURI_INVOKE("check_address", { req });
},
async getDerivations(req: GetDerivations) : Promise<GetDerivationsResponse> {
    return await TAURI_INVOKE("get_derivations", { req });
},
async getAreCoinsSpendable(req: GetAreCoinsSpendable) : Promise<GetAreCoinsSpendableResponse> {
    return await TAURI_INVOKE("get_are_coins_spendable", { req });
},
async getSpendableCoinCount(req: GetSpendableCoinCount) : Promise<GetSpendableCoinCountResponse> {
    return await TAURI_INVOKE("get_spendable_coin_count", { req });
},
async getCoinsByIds(req: GetCoinsByIds) : Promise<GetCoinsByIdsResponse> {
    return await TAURI_INVOKE("get_coins_by_ids", { req });
},
async getCoins(req: GetCoins) : Promise<GetCoinsResponse> {
    return await TAURI_INVOKE("get_coins", { req });
},
async getCats(req: GetCats) : Promise<GetCatsResponse> {
    return await TAURI_INVOKE("get_cats", { req });
},
async getAllCats(req: GetAllCats) : Promise<GetAllCatsResponse> {
    return await TAURI_INVOKE("get_all_cats", { req });
},
async getToken(req: GetToken) : Promise<GetTokenResponse> {
    return await TAURI_INVOKE("get_token", { req });
},
async getDids(req: GetDids) : Promise<GetDidsResponse> {
    return await TAURI_INVOKE("get_dids", { req });
},
async getMinterDidIds(req: GetMinterDidIds) : Promise<GetMinterDidIdsResponse> {
    return await TAURI_INVOKE("get_minter_did_ids", { req });
},
async getNftCollections(req: GetNftCollections) : Promise<GetNftCollectionsResponse> {
    return await TAURI_INVOKE("get_nft_collections", { req });
},
async getNftCollection(req: GetNftCollection) : Promise<GetNftCollectionResponse> {
    return await TAURI_INVOKE("get_nft_collection", { req });
},
async getNfts(req: GetNfts) : Promise<GetNftsResponse> {
    return await TAURI_INVOKE("get_nfts", { req });
},
async getNft(req: GetNft) : Promise<GetNftResponse> {
    return await TAURI_INVOKE("get_nft", { req });
},
async getNftData(req: GetNftData) : Promise<GetNftDataResponse> {
    return await TAURI_INVOKE("get_nft_data", { req });
},
async getNftIcon(req: GetNftIcon) : Promise<GetNftIconResponse> {
    return await TAURI_INVOKE("get_nft_icon", { req });
},
async getNftThumbnail(req: GetNftThumbnail) : Promise<GetNftThumbnailResponse> {
    return await TAURI_INVOKE("get_nft_thumbnail", { req });
},
async getPendingTransactions(req: GetPendingTransactions) : Promise<GetPendingTransactionsResponse> {
    return await TAURI_INVOKE("get_pending_transactions", { req });
},
async getTransaction(req: GetTransaction) : Promise<GetTransactionResponse> {
    return await TAURI_INVOKE("get_transaction", { req });
},
async getTransactions(req: GetTransactions) : Promise<GetTransactionsResponse> {
    return await TAURI_INVOKE("get_transactions", { req });
},
async validateAddress(address: string) : Promise<boolean> {
    return await TAURI_INVOKE("validate_address", { address });
},
async makeOffer(req: MakeOffer) : Promise<MakeOfferResponse> {
    return await TAURI_INVOKE("make_offer", { req });
},
async takeOffer(req: TakeOffer) : Promise<TakeOfferResponse> {
    return await TAURI_INVOKE("take_offer", { req });
},
async combineOffers(req: CombineOffers) : Promise<CombineOffersResponse> {
    return await TAURI_INVOKE("combine_offers", { req });
},
async viewOffer(req: ViewOffer) : Promise<ViewOfferResponse> {
    return await TAURI_INVOKE("view_offer", { req });
},
async importOffer(req: ImportOffer) : Promise<ImportOfferResponse> {
    return await TAURI_INVOKE("import_offer", { req });
},
async getOffers(req: GetOffers) : Promise<GetOffersResponse> {
    return await TAURI_INVOKE("get_offers", { req });
},
async getOffer(req: GetOffer) : Promise<GetOfferResponse> {
    return await TAURI_INVOKE("get_offer", { req });
},
async deleteOffer(req: DeleteOffer) : Promise<DeleteOfferResponse> {
    return await TAURI_INVOKE("delete_offer", { req });
},
async cancelOffer(req: CancelOffer) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("cancel_offer", { req });
},
async cancelOffers(req: CancelOffers) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("cancel_offers", { req });
},
async networkConfig() : Promise<NetworkConfig> {
    return await TAURI_INVOKE("network_config");
},
async setDiscoverPeers(req: SetDiscoverPeers) : Promise<EmptyResponse> {
    return await TAURI_INVOKE("set_discover_peers", { req });
},
async setTargetPeers(req: SetTargetPeers) : Promise<EmptyResponse> {
    return await TAURI_INVOKE("set_target_peers", { req });
},
async setNetwork(req: SetNetwork) : Promise<EmptyResponse> {
    return await TAURI_INVOKE("set_network", { req });
},
async setNetworkOverride(req: SetNetworkOverride) : Promise<EmptyResponse> {
    return await TAURI_INVOKE("set_network_override", { req });
},
async walletConfig(fingerprint: number) : Promise<Wallet | null> {
    return await TAURI_INVOKE("wallet_config", { fingerprint });
},
async defaultWalletConfig() : Promise<WalletDefaults> {
    return await TAURI_INVOKE("default_wallet_config");
},
async getNetworks(req: GetNetworks) : Promise<NetworkList> {
    return await TAURI_INVOKE("get_networks", { req });
},
async getNetwork(req: GetNetwork) : Promise<GetNetworkResponse> {
    return await TAURI_INVOKE("get_network", { req });
},
async setDeltaSync(req: SetDeltaSync) : Promise<EmptyResponse> {
    return await TAURI_INVOKE("set_delta_sync", { req });
},
async setDeltaSyncOverride(req: SetDeltaSyncOverride) : Promise<EmptyResponse> {
    return await TAURI_INVOKE("set_delta_sync_override", { req });
},
async updateCat(req: UpdateCat) : Promise<UpdateCatResponse> {
    return await TAURI_INVOKE("update_cat", { req });
},
async resyncCat(req: ResyncCat) : Promise<ResyncCatResponse> {
    return await TAURI_INVOKE("resync_cat", { req });
},
async updateDid(req: UpdateDid) : Promise<UpdateDidResponse> {
    return await TAURI_INVOKE("update_did", { req });
},
async updateNft(req: UpdateNft) : Promise<UpdateNftResponse> {
    return await TAURI_INVOKE("update_nft", { req });
},
async updateNftCollection(req: UpdateNftCollection) : Promise<UpdateNftCollectionResponse> {
    return await TAURI_INVOKE("update_nft_collection", { req });
},
async redownloadNft(req: RedownloadNft) : Promise<RedownloadNftResponse> {
    return await TAURI_INVOKE("redownload_nft", { req });
},
async increaseDerivationIndex(req: IncreaseDerivationIndex) : Promise<IncreaseDerivationIndexResponse> {
    return await TAURI_INVOKE("increase_derivation_index", { req });
},
async getPeers(req: GetPeers) : Promise<GetPeersResponse> {
    return await TAURI_INVOKE("get_peers", { req });
},
async addPeer(req: AddPeer) : Promise<EmptyResponse> {
    return await TAURI_INVOKE("add_peer", { req });
},
async removePeer(req: RemovePeer) : Promise<EmptyResponse> {
    return await TAURI_INVOKE("remove_peer", { req });
},
async filterUnlockedCoins(req: FilterUnlockedCoins) : Promise<FilterUnlockedCoinsResponse> {
    return await TAURI_INVOKE("filter_unlocked_coins", { req });
},
async getAssetCoins(req: GetAssetCoins) : Promise<SpendableCoin[]> {
    return await TAURI_INVOKE("get_asset_coins", { req });
},
async signMessageWithPublicKey(req: SignMessageWithPublicKey) : Promise<SignMessageWithPublicKeyResponse> {
    return await TAURI_INVOKE("sign_message_with_public_key", { req });
},
async signMessageByAddress(req: SignMessageByAddress) : Promise<SignMessageByAddressResponse> {
    return await TAURI_INVOKE("sign_message_by_address", { req });
},
async sendTransactionImmediately(req: SendTransactionImmediately) : Promise<SendTransactionImmediatelyResponse> {
    return await TAURI_INVOKE("send_transaction_immediately", { req });
},
async isRpcRunning() : Promise<boolean> {
    return await TAURI_INVOKE("is_rpc_running");
},
async startRpcServer() : Promise<null> {
    return await TAURI_INVOKE("start_rpc_server");
},
async stopRpcServer() : Promise<null> {
    return await TAURI_INVOKE("stop_rpc_server");
},
async getRpcRunOnStartup() : Promise<boolean> {
    return await TAURI_INVOKE("get_rpc_run_on_startup");
},
async setRpcRunOnStartup(runOnStartup: boolean) : Promise<null> {
    return await TAURI_INVOKE("set_rpc_run_on_startup", { runOnStartup });
},
async switchWallet() : Promise<null> {
    return await TAURI_INVOKE("switch_wallet");
},
async moveKey(fingerprint: number, index: number) : Promise<null> {
    return await TAURI_INVOKE("move_key", { fingerprint, index });
},
async downloadCniOffercode(code: string) : Promise<string> {
    return await TAURI_INVOKE("download_cni_offercode", { code });
},
async getLogs() : Promise<LogFile[]> {
    return await TAURI_INVOKE("get_logs");
}
}

/** user-defined events **/


export const events = __makeEvents__<{
syncEvent: SyncEvent
}>({
syncEvent: "sync-event"
})

/** user-defined constants **/



/** user-defined types **/

export type AddNftUri = { nft_id: string; uri: string; fee: Amount; kind: NftUriKind; auto_submit?: boolean }
export type AddPeer = { ip: string }
export type AddressKind = "own" | "burn" | "launcher" | "offer" | "external" | "unknown"
export type Amount = string | number
export type Asset = { asset_id: string | null; name: string | null; ticker: string | null; precision: number; icon_url: string | null; description: string | null; is_sensitive_content: boolean; is_visible: boolean; revocation_address: string | null; kind: AssetKind }
export type AssetCoinType = "cat" | "did" | "nft"
export type AssetKind = "token" | "nft" | "did"
export type Assets = { xch: Amount; cats: CatAmount[]; nfts: string[] }
export type AssignNftsToDid = { nft_ids: string[]; did_id: string | null; fee: Amount; auto_submit?: boolean }
export type AutoCombineCat = { asset_id: string; max_coins: number; max_coin_amount: Amount | null; fee: Amount; auto_submit?: boolean }
export type AutoCombineCatResponse = { coin_ids: string[]; summary: TransactionSummary; coin_spends: CoinSpendJson[] }
export type AutoCombineXch = { max_coins: number; max_coin_amount: Amount | null; fee: Amount; auto_submit?: boolean }
export type AutoCombineXchResponse = { coin_ids: string[]; summary: TransactionSummary; coin_spends: CoinSpendJson[] }
export type BulkMintNfts = { mints: NftMint[]; did_id: string; fee: Amount; auto_submit?: boolean }
export type BulkMintNftsResponse = { nft_ids: string[]; summary: TransactionSummary; coin_spends: CoinSpendJson[] }
export type BulkSendCat = { asset_id: string; addresses: string[]; amount: Amount; fee: Amount; include_hint?: boolean; memos?: string[]; auto_submit?: boolean }
export type BulkSendXch = { addresses: string[]; amount: Amount; fee: Amount; memos?: string[]; auto_submit?: boolean }
export type CancelOffer = { offer_id: string; fee: Amount; auto_submit?: boolean }
export type CancelOffers = { offer_ids: string[]; fee: Amount; auto_submit?: boolean }
export type CatAmount = { asset_id: string; amount: Amount }
export type ChangeMode = { mode: "default" } | 
/**
 * Reuse the first address of coins involved in the transaction
 * as the change address for the output. This improves compatibility
 * with wallets which do not support multiple addresses.
 */
{ mode: "same" } | 
/**
 * Use an address that has not been used before as the change address
 * for the output. This is beneficial for privacy, but results in more
 * addresses being generated and used which can make syncing slower.
 */
{ mode: "new" }
export type CheckAddress = { address: string }
export type CheckAddressResponse = { valid: boolean }
export type Coin = { parent_coin_info: string; puzzle_hash: string; amount: number }
export type CoinFilterMode = "all" | "selectable" | "owned" | "spent" | "clawback"
export type CoinJson = { parent_coin_info: string; puzzle_hash: string; amount: Amount }
export type CoinRecord = { coin_id: string; address: string; amount: Amount; transaction_id: string | null; offer_id: string | null; clawback_timestamp: number | null; created_height: number | null; spent_height: number | null; spent_timestamp: number | null; created_timestamp: number | null }
export type CoinSortMode = "coin_id" | "amount" | "created_height" | "spent_height" | "clawback_timestamp"
export type CoinSpend = { coin: Coin; puzzle_reveal: string; solution: string }
export type CoinSpendJson = { coin: CoinJson; puzzle_reveal: string; solution: string }
export type Combine = { coin_ids: string[]; fee: Amount; auto_submit?: boolean }
export type CombineOffers = { offers: string[] }
export type CombineOffersResponse = { offer: string }
export type CreateDid = { name: string; fee: Amount; auto_submit?: boolean }
export type DeleteDatabase = { fingerprint: number; network: string }
export type DeleteDatabaseResponse = Record<string, never>
export type DeleteKey = { fingerprint: number }
export type DeleteKeyResponse = Record<string, never>
export type DeleteOffer = { offer_id: string }
export type DeleteOfferResponse = Record<string, never>
export type DerivationMode = { mode: "default" } | 
/**
 * Automatically generate new addresses if there aren't enough that
 * haven't been used yet.
 */
{ mode: "auto"; derivation_batch_size: number } | 
/**
 * Don't generate any new addresses, only use existing ones.
 */
{ mode: "static" }
export type DerivationRecord = { index: number; public_key: string; address: string }
export type DidRecord = { launcher_id: string; name: string | null; visible: boolean; coin_id: string; address: string; amount: Amount; recovery_hash: string | null; created_height: number | null }
export type EmptyResponse = Record<string, never>
export type Error = { kind: ErrorKind; reason: string }
export type ErrorKind = "wallet" | "api" | "not_found" | "unauthorized" | "internal" | "database_migration" | "nfc"
export type FilterUnlockedCoins = { coin_ids: string[] }
export type FilterUnlockedCoinsResponse = { coin_ids: string[] }
export type GenerateMnemonic = { use_24_words: boolean }
export type GenerateMnemonicResponse = { mnemonic: string }
export type GetAllCats = Record<string, never>
export type GetAllCatsResponse = { cats: TokenRecord[] }
export type GetAreCoinsSpendable = { coin_ids: string[] }
export type GetAreCoinsSpendableResponse = { spendable: boolean }
export type GetAssetCoins = { type?: AssetCoinType | null; assetId?: string | null; includedLocked?: boolean | null; offset?: number | null; limit?: number | null }
export type GetCats = Record<string, never>
export type GetCatsResponse = { cats: TokenRecord[] }
export type GetCoins = { asset_id?: string | null; offset: number; limit: number; sort_mode?: CoinSortMode; filter_mode?: CoinFilterMode; ascending?: boolean }
export type GetCoinsByIds = { coin_ids: string[] }
export type GetCoinsByIdsResponse = { coins: CoinRecord[] }
export type GetCoinsResponse = { coins: CoinRecord[]; total: number }
export type GetDatabaseStats = Record<string, never>
export type GetDatabaseStatsResponse = { total_pages: number; free_pages: number; free_percentage: number; page_size: number; database_size_bytes: number; free_space_bytes: number; wal_pages: number }
export type GetDerivations = { hardened?: boolean; offset: number; limit: number }
export type GetDerivationsResponse = { derivations: DerivationRecord[]; total: number }
export type GetDids = Record<string, never>
export type GetDidsResponse = { dids: DidRecord[] }
export type GetKey = { fingerprint?: number | null }
export type GetKeyResponse = { key: KeyInfo | null }
export type GetKeys = Record<string, never>
export type GetKeysResponse = { keys: KeyInfo[] }
export type GetMinterDidIds = { offset: number; limit: number }
export type GetMinterDidIdsResponse = { did_ids: string[]; total: number }
export type GetNetwork = Record<string, never>
export type GetNetworkResponse = { network: Network; kind: NetworkKind }
export type GetNetworks = Record<string, never>
export type GetNft = { nft_id: string }
export type GetNftCollection = { collection_id: string | null }
export type GetNftCollectionResponse = { collection: NftCollectionRecord | null }
export type GetNftCollections = { offset: number; limit: number; include_hidden: boolean }
export type GetNftCollectionsResponse = { collections: NftCollectionRecord[]; total: number }
export type GetNftData = { nft_id: string }
export type GetNftDataResponse = { data: NftData | null }
export type GetNftIcon = { nft_id: string }
export type GetNftIconResponse = { icon: string | null }
export type GetNftResponse = { nft: NftRecord | null }
export type GetNftThumbnail = { nft_id: string }
export type GetNftThumbnailResponse = { thumbnail: string | null }
export type GetNfts = { collection_id?: string | null; minter_did_id?: string | null; owner_did_id?: string | null; name?: string | null; offset: number; limit: number; sort_mode: NftSortMode; include_hidden: boolean }
export type GetNftsResponse = { nfts: NftRecord[]; total: number }
export type GetOffer = { offer_id: string }
export type GetOfferResponse = { offer: OfferRecord }
export type GetOffers = Record<string, never>
export type GetOffersResponse = { offers: OfferRecord[] }
export type GetPeers = Record<string, never>
export type GetPeersResponse = { peers: PeerRecord[] }
export type GetPendingTransactions = Record<string, never>
export type GetPendingTransactionsResponse = { transactions: PendingTransactionRecord[] }
export type GetSecretKey = { fingerprint: number }
export type GetSecretKeyResponse = { secrets: SecretKeyInfo | null }
export type GetSpendableCoinCount = { asset_id: string | null }
export type GetSpendableCoinCountResponse = { count: number }
export type GetSyncStatus = Record<string, never>
export type GetSyncStatusResponse = { balance: Amount; unit: Unit; synced_coins: number; total_coins: number; receive_address: string; burn_address: string; unhardened_derivation_index: number; hardened_derivation_index: number; checked_files: number; total_files: number; database_size: number }
export type GetToken = { asset_id: string | null }
export type GetTokenResponse = { token: TokenRecord | null }
export type GetTransaction = { height: number }
export type GetTransactionResponse = { transaction: TransactionRecord | null }
export type GetTransactions = { offset: number; limit: number; ascending: boolean; find_value: string | null }
export type GetTransactionsResponse = { transactions: TransactionRecord[]; total: number }
export type GetVersion = Record<string, never>
export type GetVersionResponse = { version: string }
export type ImportKey = { name: string; key: string; derivation_index?: number; save_secrets?: boolean; login?: boolean }
export type ImportKeyResponse = { fingerprint: number }
export type ImportOffer = { offer: string }
export type ImportOfferResponse = { offer_id: string }
export type IncreaseDerivationIndex = { hardened?: boolean | null; index: number }
export type IncreaseDerivationIndexResponse = Record<string, never>
export type InheritedNetwork = "mainnet" | "testnet11"
export type IssueCat = { name: string; ticker: string; amount: Amount; fee: Amount; auto_submit?: boolean }
export type KeyInfo = { name: string; fingerprint: number; public_key: string; kind: KeyKind; has_secrets: boolean; network_id: string }
export type KeyKind = "bls"
export type LineageProof = { parentName: string | null; innerPuzzleHash: string | null; amount: number | null }
export type LogFile = { name: string; text: string }
export type Login = { fingerprint: number }
export type LoginResponse = Record<string, never>
export type Logout = Record<string, never>
export type LogoutResponse = Record<string, never>
export type MakeOffer = { requested_assets: Assets; offered_assets: Assets; fee: Amount; receive_address?: string | null; expires_at_second?: number | null; auto_import?: boolean }
export type MakeOfferResponse = { offer: string; offer_id: string }
export type Network = { name: string; ticker: string; prefix?: string | null; precision: number; network_id?: string | null; default_port: number; genesis_challenge: string; agg_sig_me?: string | null; dns_introducers: string[]; peer_introducers: string[]; inherit?: InheritedNetwork | null }
export type NetworkConfig = { default_network: string; target_peers: number; discover_peers: boolean }
export type NetworkKind = "mainnet" | "testnet" | "unknown"
export type NetworkList = { networks: Network[] }
export type NftCollectionRecord = { collection_id: string; did_id: string; metadata_collection_id: string; visible: boolean; name: string | null; icon: string | null }
export type NftData = { blob: string | null; mime_type: string | null; hash_matches: boolean; metadata_json: string | null; metadata_hash_matches: boolean }
export type NftMint = { address?: string | null; edition_number?: number | null; edition_total?: number | null; data_hash?: string | null; data_uris?: string[]; metadata_hash?: string | null; metadata_uris?: string[]; license_hash?: string | null; license_uris?: string[]; royalty_address?: string | null; royalty_ten_thousandths?: number }
export type NftRecord = { launcher_id: string; collection_id: string | null; collection_name: string | null; minter_did: string | null; owner_did: string | null; visible: boolean; sensitive_content: boolean; name: string | null; created_height: number | null; coin_id: string; address: string; royalty_address: string; royalty_ten_thousandths: number; data_uris: string[]; data_hash: string | null; metadata_uris: string[]; metadata_hash: string | null; license_uris: string[]; license_hash: string | null; edition_number: number | null; edition_total: number | null; icon_url: string | null }
export type NftRoyalty = { royalty_address: string; royalty_basis_points: number }
export type NftSortMode = "name" | "recent"
export type NftUriKind = "data" | "metadata" | "license"
export type NormalizeDids = { did_ids: string[]; fee: Amount; auto_submit?: boolean }
export type OfferAsset = { asset: Asset; amount: Amount; royalty: Amount; nft_royalty: NftRoyalty | null }
export type OfferRecord = { offer_id: string; offer: string; status: OfferRecordStatus; creation_timestamp: number; summary: OfferSummary }
export type OfferRecordStatus = "pending" | "active" | "completed" | "cancelled" | "expired"
export type OfferSummary = { fee: Amount; maker: OfferAsset[]; taker: OfferAsset[]; expiration_height: number | null; expiration_timestamp: number | null }
export type PeerRecord = { ip_addr: string; port: number; peak_height: number; user_managed: boolean }
export type PendingTransactionRecord = { transaction_id: string; fee: Amount; submitted_at: string | null }
export type PerformDatabaseMaintenance = { force_vacuum: boolean }
export type PerformDatabaseMaintenanceResponse = { vacuum_duration_ms: number; analyze_duration_ms: number; wal_checkpoint_duration_ms: number; total_duration_ms: number; pages_vacuumed: number; wal_pages_checkpointed: number }
export type RedownloadNft = { nft_id: string }
export type RedownloadNftResponse = Record<string, never>
export type RemovePeer = { ip: string; ban: boolean }
export type RenameKey = { fingerprint: number; name: string }
export type RenameKeyResponse = Record<string, never>
export type Resync = { fingerprint: number; delete_coins?: boolean; delete_assets?: boolean; delete_files?: boolean; delete_offers?: boolean; delete_addresses?: boolean; delete_blocks?: boolean }
export type ResyncCat = { asset_id: string }
export type ResyncCatResponse = Record<string, never>
export type ResyncResponse = Record<string, never>
export type SecretKeyInfo = { mnemonic: string | null; secret_key: string }
export type SendCat = { asset_id: string; address: string; amount: Amount; fee: Amount; include_hint?: boolean; memos?: string[]; clawback?: number | null; auto_submit?: boolean }
export type SendTransactionImmediately = { spend_bundle: SpendBundle }
export type SendTransactionImmediatelyResponse = { status: number; error: string | null }
export type SendXch = { address: string; amount: Amount; fee: Amount; memos?: string[]; clawback?: number | null; auto_submit?: boolean }
export type SetDeltaSync = { delta_sync: boolean }
export type SetDeltaSyncOverride = { fingerprint: number; delta_sync: boolean | null }
export type SetDiscoverPeers = { discover_peers: boolean }
export type SetNetwork = { name: string }
export type SetNetworkOverride = { fingerprint: number; name: string | null }
export type SetTargetPeers = { target_peers: number }
export type SignCoinSpends = { coin_spends: CoinSpendJson[]; auto_submit?: boolean; partial?: boolean }
export type SignCoinSpendsResponse = { spend_bundle: SpendBundleJson }
export type SignMessageByAddress = { message: string; address: string }
export type SignMessageByAddressResponse = { publicKey: string; signature: string }
export type SignMessageWithPublicKey = { message: string; publicKey: string }
export type SignMessageWithPublicKeyResponse = { signature: string }
export type SpendBundle = { coin_spends: CoinSpend[]; aggregated_signature: string }
export type SpendBundleJson = { coin_spends: CoinSpendJson[]; aggregated_signature: string }
export type SpendableCoin = { coin: Coin; coinName: string; puzzle: string; confirmedBlockIndex: number; locked: boolean; lineageProof: LineageProof | null }
export type Split = { coin_ids: string[]; output_count: number; fee: Amount; auto_submit?: boolean }
export type SubmitTransaction = { spend_bundle: SpendBundleJson }
export type SubmitTransactionResponse = Record<string, never>
export type SyncEvent = { type: "start"; ip: string } | { type: "stop" } | { type: "subscribed" } | { type: "derivation" } | { type: "coin_state" } | { type: "transaction_failed"; transaction_id: string; error: string | null } | { type: "puzzle_batch_synced" } | { type: "cat_info" } | { type: "did_info" } | { type: "nft_data" }
export type TakeOffer = { offer: string; fee: Amount; auto_submit?: boolean }
export type TakeOfferResponse = { summary: TransactionSummary; spend_bundle: SpendBundleJson; transaction_id: string }
export type TokenRecord = { asset_id: string | null; name: string | null; ticker: string | null; precision: number; description: string | null; icon_url: string | null; visible: boolean; balance: Amount; revocation_address: string | null }
export type TransactionCoinRecord = { coin_id: string; amount: Amount; address: string | null; address_kind: AddressKind; asset: Asset }
export type TransactionInput = { coin_id: string; amount: Amount; address: string; asset: Asset | null; outputs: TransactionOutput[] }
export type TransactionOutput = { coin_id: string; amount: Amount; address: string; receiving: boolean; burning: boolean }
export type TransactionRecord = { height: number; timestamp: number | null; spent: TransactionCoinRecord[]; created: TransactionCoinRecord[] }
export type TransactionResponse = { summary: TransactionSummary; coin_spends: CoinSpendJson[] }
export type TransactionSummary = { fee: Amount; inputs: TransactionInput[] }
export type TransferDids = { did_ids: string[]; address: string; fee: Amount; clawback?: number | null; auto_submit?: boolean }
export type TransferNfts = { nft_ids: string[]; address: string; fee: Amount; clawback?: number | null; auto_submit?: boolean }
export type Unit = { ticker: string; decimals: number }
export type UpdateCat = { record: TokenRecord }
export type UpdateCatResponse = Record<string, never>
export type UpdateDid = { did_id: string; name: string | null; visible: boolean }
export type UpdateDidResponse = Record<string, never>
export type UpdateNft = { nft_id: string; visible: boolean }
export type UpdateNftCollection = { collection_id: string; visible: boolean }
export type UpdateNftCollectionResponse = Record<string, never>
export type UpdateNftResponse = Record<string, never>
export type ViewCoinSpends = { coin_spends: CoinSpendJson[] }
export type ViewCoinSpendsResponse = { summary: TransactionSummary }
export type ViewOffer = { offer: string }
export type ViewOfferResponse = { offer: OfferSummary }
export type Wallet = { name: string; fingerprint: number; change: ChangeMode; derivation: DerivationMode; network?: string | null; delta_sync: boolean | null }
export type WalletDefaults = { change: ChangeMode; derivation: DerivationMode; delta_sync: boolean }

/** tauri-specta globals **/

import {
	invoke as TAURI_INVOKE,
	Channel as TAURI_CHANNEL,
} from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

type __EventObj__<T> = {
	listen: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
	once: (
		cb: TAURI_API_EVENT.EventCallback<T>,
	) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
	emit: null extends T
		? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
		: (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

export type Result<T, E> =
	| { status: "ok"; data: T }
	| { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
	mappings: Record<keyof T, string>,
) {
	return new Proxy(
		{} as unknown as {
			[K in keyof T]: __EventObj__<T[K]> & {
				(handle: __WebviewWindow__): __EventObj__<T[K]>;
			};
		},
		{
			get: (_, event) => {
				const name = mappings[event as keyof T];

				return new Proxy((() => {}) as any, {
					apply: (_, __, [window]: [__WebviewWindow__]) => ({
						listen: (arg: any) => window.listen(name, arg),
						once: (arg: any) => window.once(name, arg),
						emit: (arg: any) => window.emit(name, arg),
					}),
					get: (_, command: keyof __EventObj__<any>) => {
						switch (command) {
							case "listen":
								return (arg: any) => TAURI_API_EVENT.listen(name, arg);
							case "once":
								return (arg: any) => TAURI_API_EVENT.once(name, arg);
							case "emit":
								return (arg: any) => TAURI_API_EVENT.emit(name, arg);
						}
					},
				});
			},
		},
	);
}
