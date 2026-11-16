# Wallet kinds

Sage opens three kinds of wallets, and each RPC endpoint, WalletConnect command and Sage Apps bridge
method behaves differently for some of them:

- **BLS**: a key in the keychain. Its addresses are derived from the key, and it signs with the
  derived keys. A cold BLS wallet only has its public key, so it can build transactions but not sign
  them.
- **Vault**: a vault singleton, which is followed from its launcher. Its addresses are its P2 singleton
  puzzle hashes, numbered by nonce, which don't change when its custody does. It spends through the
  vault singleton, signed by the keychain keys attached to it as custody signers. Without a custody
  signer it can't sign.
- **Watch**: a list of addresses whose puzzle reveals are unknown. It can't sign anything.

The behavior of every RPC endpoint is in `crates/sage-rpc/src/wallet_kinds.rs`, which adds it to each
endpoint's OpenAPI description. A test fails if an endpoint in `crates/sage-api/endpoints.json` isn't
listed there, so a new endpoint must decide how it behaves for each kind.

## Errors

- A watch wallet refuses every signing request with the same wallet error: _Watch-only wallets can't
  create or sign transactions_.
- A vault without signers refuses signing requests with _No keys that sign for this vault are
  attached_.
- Something a kind can't do returns a wallet error that names the kind, such as _Vault wallets can't
  use a change address_.
- An endpoint that only works for one kind returns an error that the wallet isn't that kind, such as
  _Wallet 1234 isn't a vault_.

## RPC endpoints

| Endpoints                                                                                                                                      | BLS                                                    | Vault                                                                                                                                                                                                        | Watch                              |
| ---------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------- |
| Wallet management, keys, settings, peers, themes, contacts, handle resolution                                                                  | Doesn't depend on the open wallet                      | Same                                                                                                                                                                                                         | Same                               |
| Reading synced data: coins, tokens, NFTs, DIDs, options, transactions, offers, handles, `view_coin_spends`, `is_asset_owned`, metadata updates | Works                                                  | Works                                                                                                                                                                                                        | Works                              |
| `get_wallet_address`, `get_sync_status.receive_address`                                                                                        | Current receive address, or the change address         | First vault address, which never changes                                                                                                                                                                     | First watched address              |
| `get_derivations`                                                                                                                              | Derived addresses and synthetic public keys            | P2 singleton addresses by nonce, with no public key and no hardened addresses                                                                                                                                | Empty                              |
| `get_sync_status.*_derivation_index`                                                                                                           | Next derivation indexes                                | Number of vault addresses, and 0 hardened                                                                                                                                                                    | 0                                  |
| `increase_derivation_index`                                                                                                                    | Derives addresses                                      | Adds P2 singleton addresses up to the nonce, and refuses hardened ones                                                                                                                                       | Error                              |
| `check_address`                                                                                                                                | Derived addresses                                      | Vault addresses                                                                                                                                                                                              | Watched addresses                  |
| `set_change_address`                                                                                                                           | Sets the change address                                | Error, since change goes back to the vault                                                                                                                                                                   | Error, since it never makes change |
| `send_xch`, `bulk_send_xch`, `send_cat`, `bulk_send_cat`, `multi_send`, `combine`, `split`, `auto_combine_*`, `create_transaction`             | Signs with derived keys                                | Spends through the vault, after clear signing, with change back to the vault. One unconfirmed spend at a time                                                                                                | Error                              |
| `issue_cat`, `issue_more_cat`, NFT, DID and option actions, `finalize_clawback`, `claim_clawback`                                              | Signs with derived keys (`issue_more_cat` is an error) | Spends through the vault. Sage checks where its DIDs and options go. `bulk_mint_nfts` without `did_id` mints with the vault as minter, and `issue_cat` with `multi_issuance` lets the vault issue more later | Error                              |
| `make_offer`, `take_offer`, `cancel_offer`, `cancel_offers`                                                                                    | Signs with derived keys                                | Always pre-split, so the offer doesn't need the vault's signature. Only XCH and CATs can be offered. Taking spends through the vault, and cancelling takes the pre-split coins back                          | Error                              |
| `sign_coin_spends`                                                                                                                             | Signs for its derived keys                             | See [signing](#signing)                                                                                                                                                                                      | Error                              |
| `submit_transaction`                                                                                                                           | Submits a signed transaction                           | Same                                                                                                                                                                                                         | Same                               |
| `get_vault`, `attach_vault_signer`, `export_vault_config`, `import_vault_config`, `rebuild_vault_config`                                       | Error                                                  | Works                                                                                                                                                                                                        | Error                              |
| `start_vault_recovery`, `cancel_vault_recovery`, `finish_vault_recovery`, `rekey_vault`                                                        | Error                                                  | Works. Start and finish take an optional fee from a BLS wallet (`fee_fingerprint`), and the endpoint signs when `auto_submit` is set. See [recovery](#recovery)                                              | Error                              |
| `mint_vault`                                                                                                                                   | Mints from the open wallet                             | Error                                                                                                                                                                                                        | Error                              |
| `add_watch_addresses`, `remove_watch_addresses`                                                                                                | Error                                                  | Error                                                                                                                                                                                                        | Works                              |
| `get_secret_key`                                                                                                                               | Secrets of the keychain key with the fingerprint       | A vault's fingerprint isn't a keychain key, so none. Its signers have their own                                                                                                                              | None                               |

`get_vault` returns the vault's state (unsynced, active, recovery pending or melted), when a pending
recovery started and when it can finish (from block timestamps), its current coin and custody hash,
its config with its puzzle version, custody, recovery keys and timelock, and its attached signers and
their roles.

## Signing

A vault's custody signers only ever sign `AGG_SIG_ME` for the vault coin being spent, over the
delegated spend that clear signing checked. So when a vault signs a transaction, including one a dApp supplied
with `sign_coin_spends` or WalletConnect:

- The vault's spend is parsed with clear signing first. A spend that changes the vault's custody or
  melts it is refused, since nothing Sage makes through this path does either. Recovery and rekeying
  have their own endpoints.
- A request for a vault key to sign anything else is refused: a signature for another coin, any
  `AGG_SIG_*` kind other than `AGG_SIG_ME`, or a second spend of the vault coin whose delegated spend
  clear signing didn't check.
- `AGG_SIG_UNSAFE` is refused for every key Sage holds, as for BLS wallets.
- The SDK's signer checks that each puzzle reveal matches its coin's puzzle hash, so a spend can't
  claim to be the vault's while running another puzzle.

Messages:

- `sign_message_by_address` with a vault address signs as the vault. The vault signs a
  [`vault_message_envelope`](https://docs.rs/chia-sdk-driver) that sends the tree hash of
  `("Chia Signed Message", message)`, the same message a BLS key signs, and returns it as a
  `vaultsig` `VaultMessageProof` in `signature`. `public_key` is the vault's `vault1` id, which the
  verifier uses to look up the vault's custody hash on chain for
  `verify_vault_signed_message`. The proof can never be valid on chain, and stays valid while the
  vault's custody is unchanged, so the message should include a nonce or expiry when that matters.
- `sign_message_with_public_key` only works for BLS wallets, since a vault's addresses aren't locked to
  a key.

## Recovery

- Starting a recovery signs with the vault's recovery key, from the keychain or an entered mnemonic,
  and moves the vault to its recovery state with a memo of the new custody. Neither the vault nor its
  coins can pay a fee during recovery, so the fee comes from a BLS wallet in the same transaction.
- The vault's custody can cancel a pending recovery at any time before it's finished.
- Once the timelock has passed by block timestamps, anyone can finish it. Sage only finishes a
  recovery whose new custody includes a key in its keychain, and `get_vault` says why one can't be
  finished.
- Sync records every recovery start it sees. One this Sage instance didn't start raises an alert with
  a one-click cancel.
- Rekeying changes the custody key, the recovery key or the timelock, and always upgrades `Legacy`
  vaults to the newer puzzle version.
- Recovery and rekeying don't invalidate open pre-split offers, so the vault page warns about them
  first.

## WalletConnect

| Command                                                  | BLS                                | Vault                                                                                                      | Watch                                       |
| -------------------------------------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------- | ------------------------------------------- |
| `chip0002_getPublicKeys`                                 | Derived synthetic public keys      | Empty, since a vault's addresses aren't locked to keys. Use `chia_getAddress` and `chip0002_getAssetCoins` | Empty                                       |
| `chip0002_getAssetCoins`, `chip0002_getAssetBalance`     | Coins with standard puzzle reveals | Coins with P2 singleton puzzle reveals                                                                     | Error, since the puzzle reveals are unknown |
| `chip0002_signCoinSpends`                                | Signs                              | Signs as above                                                                                             | Refused                                     |
| `chip0002_sendTransaction`                               | Submits                            | Submits                                                                                                    | Submits                                     |
| `chip0002_signMessage`                                   | Signs                              | Error: sign by address instead                                                                             | Refused                                     |
| `chia_signMessageByAddress`                              | Signs                              | Signs as the vault, with a `vaultsig` proof                                                                | Refused                                     |
| `chia_send`                                              | Sends                              | Sends through the vault                                                                                    | Refused                                     |
| `chia_createOffer`, `chia_takeOffer`, `chia_cancelOffer` | Works                              | Pre-split offers of XCH and CATs, taken and cancelled through the vault                                    | Refused, except an off-chain cancel         |
| `chia_bulkMintNfts`                                      | Mints                              | Mints through the vault                                                                                    | Refused                                     |
| `chia_getAddress`                                        | Current receive address            | First vault address                                                                                        | First watched address                       |

The frontend refuses signing commands for read-only wallets, which are watch wallets, cold BLS wallets
and vaults without a custody signer, before asking the backend.

## Sage Apps bridge

- `wallet.listWallets` and the current-wallet methods include each wallet's kind (`bls`, `watch` or
  `vault`).
- Methods that sign are refused for watch wallets with the `watch_only` code, and for vaults without
  signers with `no_vault_signers`. `wallet.signMessage` signs by public key, so vaults refuse it with
  `vault_unsupported`. Vaults can use the other signing methods, which go through the vault as above.
- `wallet.getPublicKeys` returns the same keys as `chip0002_getPublicKeys`, so it's empty for a vault.
- `wallet.getAssetCoins` returns P2 singleton puzzle reveals for a vault's coins.
