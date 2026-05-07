use std::time::Duration;

use reqwest::Client;
use serde::Deserialize;

use crate::UriError;

#[derive(Debug, Clone, Copy)]
pub struct XchUsdPrice {
    pub usd: f64,
}

impl XchUsdPrice {
    pub async fn fetch() -> Result<Self, UriError> {
        let response = price_client()?
            .get("https://api.coingecko.com/api/v3/simple/price")
            .query(&[
                ("ids", "chia"),
                ("vs_currencies", "usd"),
            ])
            .send()
            .await?
            .error_for_status()?
            .json::<CoinGeckoSimplePriceResponse>()
            .await?;

        let usd = response
            .chia
            .usd
            .filter(|price| price.is_finite() && *price > 0.0)
            .ok_or(UriError::InvalidPriceResponse)?;

        Ok(Self { usd })
    }
}

fn price_client() -> Result<Client, UriError> {
    Ok(Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(format!(
            "{}/{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        ))
        .build()?)
}

#[derive(Debug, Deserialize)]
struct CoinGeckoSimplePriceResponse {
    chia: CoinGeckoUsdPrice,
}

#[derive(Debug, Deserialize)]
struct CoinGeckoUsdPrice {
    usd: Option<f64>,
}
