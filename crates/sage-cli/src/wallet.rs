//! Readable wallet commands over the RPC: arguments in the units people use, output people
//! can read. `sage rpc <endpoint> '<json>'` stays the raw form for everything else.

use std::io::{self, BufRead, IsTerminal, Write};

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Subcommand};
use sage_api::{
    Amount, GetCats, GetSyncStatus, GetToken, GetWallet, HandleConfirmation, RecipientError,
    RecipientErrorKind, SendCat, SendXch, SignCoinSpends, TransactionResponse, Unit, WalletKind,
};
use sage_client::{Client, SageRpcError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Args)]
pub struct WalletArgs {
    /// Print the RPC response as JSON instead of text
    #[clap(long, global = true)]
    json: bool,

    #[clap(subcommand)]
    command: WalletCommand,
}

#[derive(Debug, Subcommand)]
enum WalletCommand {
    /// The logged-in wallet, its sync progress, spendable balance and receive address
    Status,
    /// XCH and token balances
    Balance,
    /// The current receive address
    Address,
    /// Send XCH, or a token with --asset. Shows the transaction the wallet built and asks
    /// before signing it
    Send {
        /// The recipient's address, or an XCHandles handle written as `@handle`
        address: String,
        /// Amount in whole units, such as 0.25 (XCH) or 12.5 (a token)
        amount: String,
        /// Network fee in XCH
        #[clap(long, default_value = "0")]
        fee: String,
        /// Asset id of the token to send instead of XCH
        #[clap(long)]
        asset: Option<String>,
        /// A text memo for the recipient (repeatable)
        #[clap(long)]
        memo: Vec<String>,
        /// Sign and submit without asking
        #[clap(long)]
        yes: bool,
        /// The new address of a handle recipient which resolves to a different address than
        /// before, to confirm sending to it. Without this, sending to a changed handle fails
        /// and shows both addresses.
        #[clap(long, value_name = "ADDRESS")]
        confirm_handle: Option<String>,
    },
}

impl WalletArgs {
    pub async fn handle(self) -> Result<()> {
        let client = Client::new()?;
        let json = self.json;

        match self.command {
            WalletCommand::Status => status(&client, json).await,
            WalletCommand::Balance => balance(&client, json).await,
            WalletCommand::Address => {
                let sync = rpc(client.get_sync_status(GetSyncStatus {}).await)?;
                if json {
                    print_json(&sync.receive_address);
                } else {
                    println!("{}", sync.receive_address);
                }
                Ok(())
            }
            WalletCommand::Send {
                address,
                amount,
                fee,
                asset,
                memo,
                yes,
                confirm_handle,
            } => {
                send(
                    &client,
                    SendArgs {
                        address,
                        amount,
                        fee,
                        asset,
                        memos: memo,
                        yes,
                        confirm_handle,
                    },
                    json,
                )
                .await
            }
        }
    }
}

async fn status(client: &Client, json: bool) -> Result<()> {
    let wallet = rpc(client.get_wallet(GetWallet { fingerprint: None }).await)?;
    let sync = rpc(client.get_sync_status(GetSyncStatus {}).await)?;

    if json {
        print_json(&serde_json::json!({ "wallet": wallet.wallet, "sync": sync }));
        return Ok(());
    }

    let Some(wallet) = wallet.wallet else {
        bail!("No wallet is logged in");
    };

    println!("Wallet:   {} ({})", wallet.name, wallet.fingerprint);
    if matches!(wallet.kind, WalletKind::Watch { .. }) {
        println!("Kind:     watch-only");
    }
    println!("Network:  {}", wallet.network_id);
    println!(
        "Balance:  {} {} spendable",
        format_units(amount(&sync.selectable_balance)?, sync.unit.precision),
        sync.unit.ticker
    );
    println!("Synced:   {}/{} coins", sync.synced_coins, sync.total_coins);
    println!("Address:  {}", sync.receive_address);

    Ok(())
}

async fn balance(client: &Client, json: bool) -> Result<()> {
    let sync = rpc(client.get_sync_status(GetSyncStatus {}).await)?;
    let cats = rpc(client.get_cats(GetCats {}).await)?.cats;

    if json {
        print_json(&serde_json::json!({ "xch": sync.selectable_balance, "cats": cats }));
        return Ok(());
    }

    println!(
        "{:>24}  {}",
        format_units(amount(&sync.selectable_balance)?, sync.unit.precision),
        sync.unit.ticker
    );

    for cat in cats.iter().filter(|cat| cat.visible) {
        let balance = amount(&cat.balance)?;
        if balance == 0 {
            continue;
        }

        let label = cat
            .ticker
            .clone()
            .or_else(|| cat.name.clone())
            .unwrap_or_else(|| "Unknown CAT".to_string());

        println!(
            "{:>24}  {label}  {}",
            format_units(balance, cat.precision),
            cat.asset_id.as_deref().unwrap_or_default()
        );
    }

    Ok(())
}

struct SendArgs {
    address: String,
    amount: String,
    fee: String,
    asset: Option<String>,
    memos: Vec<String>,
    yes: bool,
    confirm_handle: Option<String>,
}

async fn send(client: &Client, args: SendArgs, json: bool) -> Result<()> {
    let sync = rpc(client.get_sync_status(GetSyncStatus {}).await)?;
    let fee = parse_units(&args.fee, sync.unit.precision).context("invalid --fee")?;
    let memos = args.memos.iter().map(hex::encode).collect();
    let confirmed_handles = confirmed_handles(&args)?;

    // Build the transaction without submitting it, so what is shown is what gets signed.
    let (built, ticker, precision) = if let Some(asset_id) = args.asset.clone() {
        let token = rpc(client
            .get_token(GetToken {
                asset_id: Some(asset_id.clone()),
            })
            .await)?
        .token
        .context("This wallet does not hold that token")?;
        let amount = parse_units(&args.amount, token.precision).context("invalid amount")?;
        let ticker = token
            .ticker
            .or(token.name)
            .unwrap_or_else(|| "CAT".to_string());

        let built = rpc(client
            .send_cat(SendCat {
                confirmed_handles: confirmed_handles.clone(),
                asset_id,
                address: args.address.clone(),
                amount: Amount::u64(amount),
                fee: Amount::u64(fee),
                include_hint: true,
                memos,
                clawback: None,
                coin_ids: None,
                auto_submit: false,
            })
            .await)?;
        (built, ticker, token.precision)
    } else {
        let amount = parse_units(&args.amount, sync.unit.precision).context("invalid amount")?;

        let built = rpc(client
            .send_xch(SendXch {
                confirmed_handles,
                address: args.address.clone(),
                amount: Amount::u64(amount),
                fee: Amount::u64(fee),
                memos,
                clawback: None,
                coin_ids: None,
                auto_submit: false,
            })
            .await)?;
        (built, sync.unit.ticker.clone(), sync.unit.precision)
    };

    print_preview(&built, &args, &ticker, precision, &sync.unit)?;

    if !args.yes && !confirm()? {
        bail!("Cancelled; nothing was signed");
    }

    let signed = rpc(client
        .sign_coin_spends(SignCoinSpends {
            coin_spends: built.coin_spends,
            auto_submit: true,
            partial: false,
            aggregated_signature: None,
        })
        .await)?;

    if json {
        print_json(&signed);
    } else {
        println!("Signed and submitted.");
    }

    Ok(())
}

/// The handle confirmation for `--confirm-handle`, which only applies to a handle recipient.
fn confirmed_handles(args: &SendArgs) -> Result<Vec<HandleConfirmation>> {
    let Some(address) = &args.confirm_handle else {
        return Ok(Vec::new());
    };

    let handle = args.address.trim();

    if !handle.starts_with('@') {
        bail!("--confirm-handle only applies when sending to a handle written as @handle");
    }

    Ok(vec![HandleConfirmation {
        handle: handle.to_string(),
        address: address.clone(),
    }])
}

/// What leaves the wallet, read from the transaction the wallet built rather than from the
/// arguments, so a wrong address or amount shows up before anything is signed.
fn print_preview(
    built: &TransactionResponse,
    args: &SendArgs,
    ticker: &str,
    precision: u8,
    unit: &Unit,
) -> Result<()> {
    println!("Send {} {ticker} to {}", args.amount, args.address);

    for handle in &built.summary.handles {
        let note = if handle.cached {
            " (verified earlier; it can't be verified right now)"
        } else {
            ""
        };
        println!(
            "@{} resolves to {}{note}, last updated at height {}",
            handle.handle, handle.address, handle.updated_height
        );
    }

    let mut leaving = Vec::new();
    for input in &built.summary.inputs {
        let input_precision = input
            .asset
            .as_ref()
            .map_or(precision, |asset| asset.precision);
        let input_ticker = input
            .asset
            .as_ref()
            .and_then(|asset| asset.ticker.clone())
            .unwrap_or_else(|| ticker.to_string());

        for output in input.outputs.iter().filter(|output| !output.receiving) {
            leaving.push(format!(
                "  {} {input_ticker} -> {}",
                format_units(amount(&output.amount)?, input_precision),
                output.address
            ));
        }
    }

    println!("Leaving your wallet:");
    for line in &leaving {
        println!("{line}");
    }
    println!(
        "Network fee: {} {}",
        format_units(amount(&built.summary.fee)?, unit.precision),
        unit.ticker
    );
    println!(
        "Spends {} coin(s) of yours; change returns to your wallet.",
        built.summary.inputs.len()
    );

    Ok(())
}

fn confirm() -> Result<bool> {
    if !io::stdin().is_terminal() {
        bail!("Not signing without a terminal to confirm on; pass --yes to sign anyway");
    }

    print!("Sign and submit? [y/N] ");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().lock().read_line(&mut line)?;

    Ok(matches!(line.trim(), "y" | "Y" | "yes" | "YES"))
}

fn rpc<T>(result: Result<T, SageRpcError>) -> Result<T> {
    result.map_err(|error| match error {
        SageRpcError::Api(_, message) => {
            anyhow!(recipient_error_message(&message).unwrap_or(message))
        }
        error => anyhow!(error),
    })
}

/// The body of an RPC error caused by recipients which couldn't be used.
#[derive(Deserialize)]
struct RecipientErrorBody {
    recipients: Vec<RecipientError>,
}

/// Lists each recipient which couldn't be used, and how to confirm a handle which changed.
fn recipient_error_message(body: &str) -> Option<String> {
    let body: RecipientErrorBody = serde_json::from_str(body).ok()?;
    let mut lines = Vec::new();

    for recipient in body.recipients {
        lines.push(recipient.reason);

        if recipient.kind == RecipientErrorKind::Changed
            && let Some(address) = recipient.address
        {
            lines.push(format!(
                "If you trust the new address, send again with --confirm-handle {address}"
            ));
        }
    }

    Some(lines.join("\n"))
}

fn amount(value: &Amount) -> Result<u128> {
    value
        .to_u128()
        .with_context(|| format!("unexpected amount from the wallet: {value:?}"))
}

fn print_json<T: Serialize>(value: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("could not serialize result")
    );
}

/// Whole units (`1.25`) to base units at `precision` decimals, exactly: no floating point,
/// and more decimals than the asset has is an error rather than a silent rounding.
pub fn parse_units(input: &str, precision: u8) -> Result<u64> {
    let text = input.trim();
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));

    if (whole.is_empty() && fraction.is_empty())
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        bail!("{input:?} is not an amount");
    }

    if fraction.len() > usize::from(precision) {
        bail!("{input} has more than {precision} decimal places");
    }

    let scale = 10u64
        .checked_pow(u32::from(precision))
        .context("precision too large")?;
    let whole: u64 = if whole.is_empty() {
        0
    } else {
        whole.parse().context("amount too large")?
    };
    let fraction: u64 = if fraction.is_empty() {
        0
    } else {
        format!("{fraction:0<width$}", width = usize::from(precision)).parse()?
    };

    whole
        .checked_mul(scale)
        .and_then(|value| value.checked_add(fraction))
        .context("amount too large")
}

/// Base units to whole units, without trailing zeros (`1250000000000` at 12 is `1.25`).
pub fn format_units(value: u128, precision: u8) -> String {
    let scale = 10u128.pow(u32::from(precision));
    let whole = value / scale;
    let fraction = value % scale;

    if fraction == 0 {
        return whole.to_string();
    }

    let fraction = format!("{fraction:0>width$}", width = usize::from(precision));
    format!("{whole}.{}", fraction.trim_end_matches('0'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explains_how_to_confirm_a_changed_handle() {
        let body = serde_json::json!({
            "kind": "recipient",
            "reason": "failed",
            "recipients": [{
                "recipient": "@alice",
                "kind": "changed",
                "reason": "@alice changed",
                "handle": "alice",
                "address": "xch1new",
                "previous_address": "xch1old",
            }],
        });

        assert_eq!(
            recipient_error_message(&body.to_string()).unwrap(),
            "@alice changed\nIf you trust the new address, send again with --confirm-handle xch1new"
        );
        assert_eq!(recipient_error_message("Wrong address prefix"), None);
    }

    #[test]
    fn parses_whole_units_exactly() {
        assert_eq!(parse_units("1", 12).unwrap(), 1_000_000_000_000);
        assert_eq!(parse_units("0.25", 12).unwrap(), 250_000_000_000);
        assert_eq!(parse_units(".5", 3).unwrap(), 500);
        assert_eq!(parse_units("12.345", 3).unwrap(), 12_345);
        assert_eq!(parse_units("7.", 3).unwrap(), 7_000);
        assert_eq!(parse_units("0.000000000001", 12).unwrap(), 1);
        assert_eq!(parse_units(" 3 ", 0).unwrap(), 3);
    }

    #[test]
    fn refuses_what_it_would_have_to_round() {
        assert!(parse_units("0.0000000000001", 12).is_err());
        assert!(parse_units("1.2345", 3).is_err());
        assert!(parse_units("1.5", 0).is_err());
    }

    #[test]
    fn refuses_what_is_not_an_amount() {
        for bad in [
            "", ".", "-1", "+1", "1e3", "1,5", "0x10", "1.2.3", "abc", "1 000",
        ] {
            assert!(parse_units(bad, 12).is_err(), "{bad:?}");
        }
        assert!(parse_units("18446744073709551616", 0).is_err());
        assert!(parse_units("18446744073.709551616", 12).is_err());
    }

    #[test]
    fn formats_without_trailing_zeros() {
        assert_eq!(format_units(1_250_000_000_000, 12), "1.25");
        assert_eq!(format_units(1_000_000_000_000, 12), "1");
        assert_eq!(format_units(1, 12), "0.000000000001");
        assert_eq!(format_units(12_345, 3), "12.345");
        assert_eq!(format_units(0, 3), "0");
        assert_eq!(format_units(5, 0), "5");
    }

    #[test]
    fn round_trips() {
        for (text, precision) in [("0.25", 12), ("12.345", 3), ("1", 12), ("0.001", 3)] {
            let units = parse_units(text, precision).unwrap();
            assert_eq!(format_units(u128::from(units), precision), text);
        }
    }
}
