use anyhow::Result;
use sage_api::{GetDerivations, GetSecretKey, Login};
use sage_permissions::{
    ApprovalBinding, Caller, CallerId, UserBridgeCapability as Cap, WalletScope,
};

use super::TestApp;
use crate::RpcSession;

fn caller(fingerprint: u32, capabilities: &[Cap]) -> Caller {
    Caller {
        id: CallerId::App("test-app".into()),
        capabilities: capabilities.iter().copied().collect(),
        wallet_scope: WalletScope::SelectedWallets {
            fingerprints: vec![fingerprint],
        },
    }
}

#[tokio::test]
async fn scoped_wallet_summaries_execute_without_granting_raw_records() -> Result<()> {
    let mut app = TestApp::new().await?;
    let fingerprint = app.setup_bls(100).await?;
    app.login(Login { fingerprint }).await?;
    app.wait_for_coins().await;
    let mut rpc = RpcSession::scoped(
        app.sage.lock().await,
        caller(
            fingerprint,
            &[Cap::WalletGetAssetBalance, Cap::WalletGetPublicKeys],
        ),
        String::new(),
        None,
    );
    let (confirmed, spendable, count) = rpc.get_asset_balance(None, None).await?;
    assert_eq!(confirmed, "100");
    assert_eq!(spendable, "100");
    assert!(count > 0);
    let request = GetDerivations {
        hardened: false,
        offset: 0,
        limit: 1,
    };
    assert!(!rpc.get_public_keys(request).await?.is_empty());
    assert!(rpc.get_derivations(request).await.is_err());
    Ok(())
}

#[tokio::test]
async fn approved_secret_access_executes_once_for_the_reviewed_wallet() -> Result<()> {
    let mut app = TestApp::new().await?;
    let fingerprint = app.setup_bls(0).await?;
    let caller = caller(fingerprint, &[Cap::WalletGetSecretKey]);
    let approval = ApprovalBinding::new(
        caller.id.clone(),
        "reviewed".into(),
        Some(fingerprint),
        u64::MAX,
    );
    let mut rpc = RpcSession::scoped(
        app.sage.lock().await,
        caller,
        "reviewed".into(),
        Some(approval),
    );
    let request = GetSecretKey { fingerprint };
    rpc.bind_approved_request("get_secret_key", &request)?;
    assert!(rpc.get_secret_key(request)?.secrets.is_some());
    assert!(rpc.get_secret_key(request).is_err());
    Ok(())
}
