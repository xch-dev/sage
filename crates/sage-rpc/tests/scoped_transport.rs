use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use chia_wallet_sdk::utils::Address;
use sage_permissions::{WalletCapability as Cap, WalletScope};
use sage_rpc::{ConnectionGrant, ConnectionKind, CredentialStore, scoped_connection_router};
use serde_json::{Value, json};
use tokio::sync::Mutex;
use tower::ServiceExt;

fn nft_id(byte: u8) -> String {
    Address::new([byte; 32].into(), "nft".into())
        .encode()
        .unwrap()
}
fn router(root: &std::path::Path) -> (axum::Router, String) {
    let store = CredentialStore::open(root).unwrap();
    let issued = store
        .issue(ConnectionGrant {
            name: "Test client".into(),
            kind: ConnectionKind::Service,
            capabilities: [
                Cap::WalletGetUserTheme,
                Cap::WalletGetUserThemes,
                Cap::WalletGetVersion,
                Cap::WalletSignCoinSpends,
            ]
            .into(),
            wallet_scope: WalletScope::AllWallets,
        })
        .unwrap();
    (
        scoped_connection_router(
            Arc::new(Mutex::new(sage::Sage::new(root, true))),
            store,
            9258,
        ),
        issued.token,
    )
}
async fn call(
    router: &axum::Router,
    token: &str,
    method: &str,
    body: Value,
) -> (StatusCode, Value) {
    let request = Request::post(format!("/{method}"))
        .header("host", "localhost:9258")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(Body::from(body.to_string()))
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 3 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes)
            .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned())),
    )
}
#[tokio::test]
async fn theme_reads_reject_paths_and_preserve_valid_missing_and_empty_ids() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("theme.json"), "private outside content").unwrap();
    let id = nft_id(1);
    let theme = root.path().join("themes").join(&id);
    std::fs::create_dir_all(&theme).unwrap();
    std::fs::write(theme.join("theme.json"), "legitimate theme").unwrap();
    let (router, token) = router(root.path());
    for malicious in [
        outside.path().to_str().unwrap(),
        "../outside",
        "not-an-nft",
        "..\\outside",
    ] {
        let (status, body) = call(
            &router,
            &token,
            "get_user_theme",
            json!({"nft_id":malicious}),
        )
        .await;
        assert_ne!(status, StatusCode::OK);
        assert!(!body.to_string().contains("private outside content"));
    }
    for (id, expected) in [
        (id, json!("legitimate theme")),
        (nft_id(2), Value::Null),
        (String::new(), Value::Null),
    ] {
        let (status, body) = call(&router, &token, "get_user_theme", json!({"nft_id":id})).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["theme"], expected);
    }
    let (status, body) = call(&router, &token, "get_user_themes", json!({})).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["themes"], json!(["legitimate theme"]));
    let sage = sage::Sage::new(root.path(), true);
    let bad = outside.path().to_str().unwrap().to_owned();
    assert!(
        sage.delete_user_theme(sage_api::DeleteUserTheme {
            nft_id: bad.clone()
        })
        .await
        .is_err()
    );
    assert!(
        sage.save_user_theme(sage_api::SaveUserTheme { nft_id: bad })
            .await
            .is_err()
    );
    assert_eq!(
        std::fs::read_to_string(outside.path().join("theme.json")).unwrap(),
        "private outside content"
    );
}
#[cfg(unix)]
#[tokio::test]
async fn theme_reads_and_lists_reject_escaping_symlinks() {
    use std::os::unix::fs::symlink;
    for escape in ["directory", "file", "root"] {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let id = nft_id(3);
        let themes = root.path().join("themes");
        let directory = themes.join(&id);
        std::fs::write(outside.path().join("theme.json"), "private outside content").unwrap();
        match escape {
            "directory" => {
                std::fs::create_dir_all(&themes).unwrap();
                symlink(outside.path(), &directory).unwrap();
            }
            "file" => {
                std::fs::create_dir_all(&directory).unwrap();
                symlink(
                    outside.path().join("theme.json"),
                    directory.join("theme.json"),
                )
                .unwrap();
            }
            "root" => {
                std::fs::create_dir_all(outside.path().join(&id)).unwrap();
                std::fs::write(
                    outside.path().join(&id).join("theme.json"),
                    "private outside content",
                )
                .unwrap();
                symlink(outside.path(), &themes).unwrap();
            }
            _ => unreachable!(),
        }
        if escape != "root" {
            let safe = themes.join(nft_id(4));
            std::fs::create_dir_all(&safe).unwrap();
            std::fs::write(safe.join("theme.json"), "safe theme").unwrap();
        }
        let sage = sage::Sage::new(root.path(), true);
        assert!(
            sage.delete_user_theme(sage_api::DeleteUserTheme { nft_id: id.clone() })
                .await
                .is_err()
        );
        assert!(
            sage.save_user_theme(sage_api::SaveUserTheme { nft_id: id.clone() })
                .await
                .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(outside.path().join("theme.json")).unwrap(),
            "private outside content"
        );
        let (router, token) = router(root.path());
        if escape != "root" {
            let (status, body) = call(&router, &token, "get_user_themes", json!({})).await;
            assert_eq!(status, StatusCode::OK);
            assert_eq!(body["themes"], json!(["safe theme"]));
        }
        for (method, request) in [
            ("get_user_theme", json!({"nft_id":id})),
            ("get_user_themes", json!({})),
        ] {
            let (_, body) = call(&router, &token, method, request).await;
            assert!(
                !body.to_string().contains("private outside content"),
                "{escape} {method}"
            );
        }
    }
}
#[tokio::test]
async fn pairing_and_rpc_have_independent_body_limits() {
    let root = tempfile::tempdir().unwrap();
    let (router, token) = router(root.path());
    let (status, _) = call(
        &router,
        &token,
        "get_version",
        json!({"padding":"a".repeat(17000)}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(&router, &token, "sign_coin_spends", json!({"coin_spends":[],"partial":false,"auto_submit":false,"aggregated_signature":"a".repeat(17000)})).await;
    // Parsed by RPC and denied for missing active wallet, rather than rejected by the pairing limit.
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = call(&router, &token, "pairing/request", json!({"grant":{"name":"a".repeat(17000),"kind":"service","capabilities":["wallet.get_version"],"walletScope":{"kind":"allWallets"}},"secretHash":"a".repeat(64)})).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    let (status, _) = call(
        &router,
        &token,
        "get_version",
        json!({"padding":"a".repeat(2 * 1024 * 1024)}),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
}
