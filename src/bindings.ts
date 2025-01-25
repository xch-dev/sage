
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
async combineXch(req: CombineXch) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("combine_xch", { req });
},
async splitXch(req: SplitXch) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("split_xch", { req });
},
async sendCat(req: SendCat) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("send_cat", { req });
},
async bulkSendCat(req: BulkSendCat) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("bulk_send_cat", { req });
},
async combineCat(req: CombineCat) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("combine_cat", { req });
},
async splitCat(req: SplitCat) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("split_cat", { req });
},
async issueCat(req: IssueCat) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("issue_cat", { req });
},
async createDid(req: CreateDid) : Promise<TransactionResponse> {
    return await TAURI_INVOKE("create_did", { req });
},
async bulkMintNfts(req: BulkMintNfts) : Promise<TransactionResponse> {
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
async getDerivations(req: GetDerivations) : Promise<GetDerivationsResponse> {
    return await TAURI_INVOKE("get_derivations", { req });
},
async getXchCoins(req: GetXchCoins) : Promise<GetXchCoinsResponse> {
    return await TAURI_INVOKE("get_xch_coins", { req });
},
async getCatCoins(req: GetCatCoins) : Promise<GetCatCoinsResponse> {
    return await TAURI_INVOKE("get_cat_coins", { req });
},
async getCats(req: GetCats) : Promise<GetCatsResponse> {
    return await TAURI_INVOKE("get_cats", { req });
},
async getCat(req: GetCat) : Promise<GetCatResponse> {
    return await TAURI_INVOKE("get_cat", { req });
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
async getPendingTransactions(req: GetPendingTransactions) : Promise<GetPendingTransactionsResponse> {
    return await TAURI_INVOKE("get_pending_transactions", { req });
},
async getTransactions(req: GetTransactions) : Promise<GetTransactionsResponse> {
    return await TAURI_INVOKE("get_transactions", { req });
},
async getTransaction(req: GetTransaction) : Promise<GetTransactionResponse> {
    return await TAURI_INVOKE("get_transaction", { req });
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
async networkConfig() : Promise<NetworkConfig> {
    return await TAURI_INVOKE("network_config");
},
async setDiscoverPeers(req: SetDiscoverPeers) : Promise<SetDiscoverPeersResponse> {
    return await TAURI_INVOKE("set_discover_peers", { req });
},
async setTargetPeers(req: SetTargetPeers) : Promise<SetTargetPeersResponse> {
    return await TAURI_INVOKE("set_target_peers", { req });
},
async setNetworkId(req: SetNetworkId) : Promise<SetNetworkIdResponse> {
    return await TAURI_INVOKE("set_network_id", { req });
},
async walletConfig(fingerprint: number) : Promise<WalletConfig> {
    return await TAURI_INVOKE("wallet_config", { fingerprint });
},
async setDeriveAutomatically(req: SetDeriveAutomatically) : Promise<SetDeriveAutomaticallyResponse> {
    return await TAURI_INVOKE("set_derive_automatically", { req });
},
async setDerivationBatchSize(req: SetDerivationBatchSize) : Promise<SetDerivationBatchSizeResponse> {
    return await TAURI_INVOKE("set_derivation_batch_size", { req });
},
async getNetworks(req: GetNetworks) : Promise<GetNetworksResponse> {
    return await TAURI_INVOKE("get_networks", { req });
},
async updateCat(req: UpdateCat) : Promise<UpdateCatResponse> {
    return await TAURI_INVOKE("update_cat", { req });
},
async removeCat(req: RemoveCat) : Promise<RemoveCatResponse> {
    return await TAURI_INVOKE("remove_cat", { req });
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
async addPeer(req: AddPeer) : Promise<AddPeerResponse> {
    return await TAURI_INVOKE("add_peer", { req });
},
async removePeer(req: RemovePeer) : Promise<RemovePeerResponse> {
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
export type AddPeerResponse = Record<string, never>
export type AddressKind = "own" | "burn" | "launcher" | "offer" | "external" | "unknown"
export type Amount = string | number
export type AssetCoinType = "cat" | "did" | "nft"
export type Assets = { xch: Amount; cats: CatAmount[]; nfts: string[] }
export type AssignNftsToDid = { nft_ids: string[]; did_id: string | null; fee: Amount; auto_submit?: boolean }
export type BulkMintNfts = { mints: NftMint[]; did_id: string; fee: Amount; auto_submit?: boolean }
export type BulkSendCat = { asset_id: string; addresses: string[]; amount: Amount; fee: Amount; memos?: string[]; auto_submit?: boolean }
export type BulkSendXch = { addresses: string[]; amount: Amount; fee: Amount; memos?: string[]; auto_submit?: boolean }
export type CancelOffer = { offer_id: string; fee: Amount; auto_submit?: boolean }
export type CatAmount = { asset_id: string; amount: Amount }
export type CatRecord = { asset_id: string; name: string | null; ticker: string | null; description: string | null; icon_url: string | null; visible: boolean; balance: Amount }
export type Coin = { parent_coin_info: string; puzzle_hash: string; amount: number }
export type CoinJson = { parent_coin_info: string; puzzle_hash: string; amount: Amount }
export type CoinRecord = { coin_id: string; address: string; amount: Amount; created_height: number | null; spent_height: number | null; create_transaction_id: string | null; spend_transaction_id: string | null; offer_id: string | null }
export type CoinSpend = { coin: Coin; puzzle_reveal: string; solution: string }
export type CoinSpendJson = { coin: CoinJson; puzzle_reveal: string; solution: string }
export type CombineCat = { coin_ids: string[]; fee: Amount; auto_submit?: boolean }
export type CombineOffers = { offers: string[] }
export type CombineOffersResponse = { offer: string }
export type CombineXch = { coin_ids: string[]; fee: Amount; auto_submit?: boolean }
export type CreateDid = { name: string; fee: Amount; auto_submit?: boolean }
export type DeleteKey = { fingerprint: number }
export type DeleteKeyResponse = Record<string, never>
export type DeleteOffer = { offer_id: string }
export type DeleteOfferResponse = Record<string, never>
export type DerivationRecord = { index: number; public_key: string; address: string }
export type DidRecord = { launcher_id: string; name: string | null; visible: boolean; coin_id: string; address: string; amount: Amount; recovery_hash: string | null; created_height: number | null; create_transaction_id: string | null }
export type Error = { kind: ErrorKind; reason: string }
export type ErrorKind = "wallet" | "api" | "not_found" | "unauthorized" | "internal"
export type FilterUnlockedCoins = { coin_ids: string[] }
export type FilterUnlockedCoinsResponse = { coin_ids: string[] }
export type GenerateMnemonic = { use_24_words: boolean }
export type GenerateMnemonicResponse = { mnemonic: string }
export type GetAssetCoins = { type?: AssetCoinType | null; assetId?: string | null; includedLocked?: boolean | null; offset?: number | null; limit?: number | null }
export type GetCat = { asset_id: string }
export type GetCatCoins = { asset_id: string }
export type GetCatCoinsResponse = { coins: CoinRecord[] }
export type GetCatResponse = { cat: CatRecord | null }
export type GetCats = Record<string, never>
export type GetCatsResponse = { cats: CatRecord[] }
export type GetDerivations = { hardened?: boolean; offset: number; limit: number }
export type GetDerivationsResponse = { derivations: DerivationRecord[] }
export type GetDids = Record<string, never>
export type GetDidsResponse = { dids: DidRecord[] }
export type GetKey = { fingerprint?: number | null }
export type GetKeyResponse = { key: KeyInfo | null }
export type GetKeys = Record<string, never>
export type GetKeysResponse = { keys: KeyInfo[] }
export type GetMinterDidIds = Record<string, never>
export type GetMinterDidIdsResponse = { did_ids: string[] }
export type GetNetworks = Record<string, never>
export type GetNetworksResponse = { networks: { [key in string]: Network } }
export type GetNft = { nft_id: string }
export type GetNftCollection = { collection_id: string | null }
export type GetNftCollectionResponse = { collection: NftCollectionRecord | null }
export type GetNftCollections = { offset: number; limit: number; include_hidden: boolean }
export type GetNftCollectionsResponse = { collections: NftCollectionRecord[] }
export type GetNftData = { nft_id: string }
export type GetNftDataResponse = { data: NftData | null }
export type GetNftResponse = { nft: NftRecord | null }
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
export type GetSyncStatus = Record<string, never>
export type GetSyncStatusResponse = { balance: Amount; unit: Unit; synced_coins: number; total_coins: number; receive_address: string; burn_address: string }
export type GetTransaction = { height: number }
export type GetTransactionResponse = { transaction: TransactionRecord }
export type GetTransactions = { offset: number; limit: number }
export type GetTransactionsResponse = { transactions: TransactionRecord[]; total: number }
export type GetXchCoins = Record<string, never>
export type GetXchCoinsResponse = { coins: CoinRecord[] }
export type ImportKey = { name: string; key: string; derivation_index?: number; save_secrets?: boolean; login?: boolean }
export type ImportKeyResponse = { fingerprint: number }
export type ImportOffer = { offer: string }
export type ImportOfferResponse = Record<string, never>
export type IncreaseDerivationIndex = { hardened: boolean; index: number }
export type IncreaseDerivationIndexResponse = Record<string, never>
export type IssueCat = { name: string; ticker: string; amount: Amount; fee: Amount; auto_submit?: boolean }
export type KeyInfo = { name: string; fingerprint: number; public_key: string; kind: KeyKind; has_secrets: boolean }
export type KeyKind = "bls"
export type LineageProof = { parentName: string | null; innerPuzzleHash: string | null; amount: number | null }
export type Login = { fingerprint: number }
export type LoginResponse = Record<string, never>
export type Logout = Record<string, never>
export type LogoutResponse = Record<string, never>
export type MakeOffer = { requested_assets: Assets; offered_assets: Assets; fee: Amount; receive_address?: string | null; expires_at_second?: number | null }
export type MakeOfferResponse = { offer: string; offer_id: string }
export type Network = { default_port: number; ticker: string; address_prefix: string; precision: number; genesis_challenge: string; agg_sig_me: string; dns_introducers: string[] }
export type NetworkConfig = { network_id: string; target_peers: number; discover_peers: boolean }
export type NftCollectionRecord = { collection_id: string; did_id: string; metadata_collection_id: string; visible: boolean; name: string | null; icon: string | null }
export type NftData = { blob: string | null; mime_type: string | null; hash_matches: boolean; metadata_json: string | null; metadata_hash_matches: boolean }
export type NftMint = { edition_number: number | null; edition_total: number | null; data_uris: string[]; metadata_uris: string[]; license_uris: string[]; royalty_address: string | null; royalty_ten_thousandths: number }
export type NftRecord = { launcher_id: string; collection_id: string | null; collection_name: string | null; minter_did: string | null; owner_did: string | null; visible: boolean; sensitive_content: boolean; name: string | null; created_height: number | null; coin_id: string; address: string; royalty_address: string; royalty_ten_thousandths: number; data_uris: string[]; data_hash: string | null; metadata_uris: string[]; metadata_hash: string | null; license_uris: string[]; license_hash: string | null; edition_number: number | null; edition_total: number | null }
export type NftSortMode = "name" | "recent"
export type NftUriKind = "data" | "metadata" | "license"
export type NormalizeDids = { did_ids: string[]; fee: Amount; auto_submit?: boolean }
export type OfferAssets = { xch: OfferXch; cats: { [key in string]: OfferCat }; nfts: { [key in string]: OfferNft } }
export type OfferCat = { amount: Amount; royalty: Amount; name: string | null; ticker: string | null; icon_url: string | null }
export type OfferNft = { image_data: string | null; image_mime_type: string | null; name: string | null; royalty_ten_thousandths: number; royalty_address: string }
export type OfferRecord = { offer_id: string; offer: string; status: OfferRecordStatus; creation_date: string; summary: OfferSummary }
export type OfferRecordStatus = "active" | "completed" | "cancelled" | "expired"
export type OfferSummary = { fee: Amount; maker: OfferAssets; taker: OfferAssets }
export type OfferXch = { amount: Amount; royalty: Amount }
export type PeerRecord = { ip_addr: string; port: number; peak_height: number }
export type PendingTransactionRecord = { transaction_id: string; fee: Amount; submitted_at: string | null }
export type RedownloadNft = { nft_id: string }
export type RedownloadNftResponse = Record<string, never>
export type RemoveCat = { asset_id: string }
export type RemoveCatResponse = Record<string, never>
export type RemovePeer = { ip: string; ban: boolean }
export type RemovePeerResponse = Record<string, never>
export type RenameKey = { fingerprint: number; name: string }
export type RenameKeyResponse = Record<string, never>
export type Resync = { fingerprint: number; delete_offer_files?: boolean; delete_unhardened_derivations?: boolean; delete_hardened_derivations?: boolean }
export type ResyncResponse = Record<string, never>
export type SecretKeyInfo = { mnemonic: string | null; secret_key: string }
export type SendCat = { asset_id: string; address: string; amount: Amount; fee: Amount; memos?: string[]; auto_submit?: boolean }
export type SendTransactionImmediately = { spend_bundle: SpendBundle }
export type SendTransactionImmediatelyResponse = { status: number; error: string | null }
export type SendXch = { address: string; amount: Amount; fee: Amount; memos?: string[]; auto_submit?: boolean }
export type SetDerivationBatchSize = { fingerprint: number; derivation_batch_size: number }
export type SetDerivationBatchSizeResponse = Record<string, never>
export type SetDeriveAutomatically = { fingerprint: number; derive_automatically: boolean }
export type SetDeriveAutomaticallyResponse = Record<string, never>
export type SetDiscoverPeers = { discover_peers: boolean }
export type SetDiscoverPeersResponse = Record<string, never>
export type SetNetworkId = { network_id: string }
export type SetNetworkIdResponse = Record<string, never>
export type SetTargetPeers = { target_peers: number }
export type SetTargetPeersResponse = Record<string, never>
export type SignCoinSpends = { coin_spends: CoinSpendJson[]; auto_submit?: boolean; partial?: boolean }
export type SignCoinSpendsResponse = { spend_bundle: SpendBundleJson }
export type SignMessageByAddress = { message: string; address: string }
export type SignMessageByAddressResponse = { publicKey: string; signature: string }
export type SignMessageWithPublicKey = { message: string; publicKey: string }
export type SignMessageWithPublicKeyResponse = { signature: string }
export type SpendBundle = { coin_spends: CoinSpend[]; aggregated_signature: string }
export type SpendBundleJson = { coin_spends: CoinSpendJson[]; aggregated_signature: string }
export type SpendableCoin = { coin: Coin; coinName: string; puzzle: string; confirmedBlockIndex: number; locked: boolean; lineageProof: LineageProof | null }
export type SplitCat = { coin_ids: string[]; output_count: number; fee: Amount; auto_submit?: boolean }
export type SplitXch = { coin_ids: string[]; output_count: number; fee: Amount; auto_submit?: boolean }
export type SubmitTransaction = { spend_bundle: SpendBundleJson }
export type SubmitTransactionResponse = Record<string, never>
export type SyncEvent = { type: "start"; ip: string } | { type: "stop" } | { type: "subscribed" } | { type: "derivation" } | { type: "coin_state" } | { type: "puzzle_batch_synced" } | { type: "cat_info" } | { type: "did_info" } | { type: "nft_data" }
export type TakeOffer = { offer: string; fee: Amount; auto_submit?: boolean }
export type TakeOfferResponse = { summary: TransactionSummary; spend_bundle: SpendBundleJson; transaction_id: string }
export type TransactionCoin = ({ type: "unknown" } | { type: "xch" } | { type: "launcher" } | { type: "cat"; asset_id: string; name: string | null; ticker: string | null; icon_url: string | null } | { type: "did"; launcher_id: string; name: string | null } | { type: "nft"; launcher_id: string; image_data: string | null; image_mime_type: string | null; name: string | null }) & { coin_id: string; amount: Amount; address: string | null; address_kind: AddressKind }
export type TransactionInput = ({ type: "unknown" } | { type: "xch" } | { type: "launcher" } | { type: "cat"; asset_id: string; name: string | null; ticker: string | null; icon_url: string | null } | { type: "did"; launcher_id: string; name: string | null } | { type: "nft"; launcher_id: string; image_data: string | null; image_mime_type: string | null; name: string | null }) & { coin_id: string; amount: Amount; address: string; outputs: TransactionOutput[] }
export type TransactionOutput = { coin_id: string; amount: Amount; address: string; receiving: boolean; burning: boolean }
export type TransactionRecord = { height: number; spent: TransactionCoin[]; created: TransactionCoin[] }
export type TransactionResponse = { summary: TransactionSummary; coin_spends: CoinSpendJson[] }
export type TransactionSummary = { fee: Amount; inputs: TransactionInput[] }
export type TransferDids = { did_ids: string[]; address: string; fee: Amount; auto_submit?: boolean }
export type TransferNfts = { nft_ids: string[]; address: string; fee: Amount; auto_submit?: boolean }
export type Unit = { ticker: string; decimals: number }
export type UpdateCat = { record: CatRecord }
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
export type WalletConfig = { name: string; derive_automatically: boolean; derivation_batch_size: number }

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
