use std::sync::Arc;

use anyhow::Result;
use rustls::{ServerConfig, pki_types::PrivateKeyDer};

use crate::cert_verifier::WalletCertVerifier;

pub(crate) fn load_rustls_config(cert_path: &str, key_path: &str) -> Result<ServerConfig> {
    use anyhow::anyhow;
    use std::fs;

    let certs = {
        let cert_file = fs::read(cert_path)?;
        rustls_pemfile::certs(&mut cert_file.as_slice())
            .map(|item| item.map_err(|_| anyhow!("Failed to parse certificate")))
            .collect::<Result<Vec<_>>>()?
    };

    if certs.is_empty() {
        anyhow::bail!("No certificates found in {cert_path}");
    }

    let mut private_keys = {
        let key_file = fs::read(key_path)?;
        rustls_pemfile::pkcs8_private_keys(&mut key_file.as_slice())
            .map(|item| item.map_err(|_| anyhow!("Failed to parse key")))
            .collect::<Result<Vec<_>>>()?
    };

    if private_keys.is_empty() {
        anyhow::bail!("No private keys found in {key_path}");
    }

    let client_cert_verifier = Arc::new(WalletCertVerifier {
        wallet_cert: certs[0].as_ref().to_vec(),
    });

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(certs, PrivateKeyDer::Pkcs8(private_keys.remove(0)))?;

    Ok(config)
}
