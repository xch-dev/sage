use anyhow::{anyhow, Result};
use rustls::{
    client::danger::HandshakeSignatureValid,
    crypto::{aws_lc_rs::default_provider, verify_tls12_signature, verify_tls13_signature},
    pki_types::{CertificateDer, PrivateKeyDer, UnixTime},
    server::{
        danger::{ClientCertVerified, ClientCertVerifier},
        ServerConfig,
    },
    DistinguishedName, SignatureScheme,
};
use std::{fs, sync::Arc};

// Custom certificate verifier that only accepts the wallet certificate
#[derive(Debug)]
struct WalletCertVerifier {
    wallet_cert: Vec<u8>,
}

impl ClientCertVerifier for WalletCertVerifier {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<ClientCertVerified, rustls::Error> {
        // Check if the presented certificate matches our wallet certificate
        if end_entity.as_ref() == self.wallet_cert {
            Ok(ClientCertVerified::assertion())
        } else {
            Err(rustls::Error::General(
                "Client certificate not allowed".into(),
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        // Delegate to the default verifier
        verify_tls12_signature(
            message,
            cert,
            dss,
            &default_provider().signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        // Delegate to the default verifier
        verify_tls13_signature(
            message,
            cert,
            dss,
            &default_provider().signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

pub fn load_rustls_config(cert_path: &str, key_path: &str) -> Result<ServerConfig> {
    // Load the certificate
    let certs = {
        let cert_file = fs::read(cert_path)?;
        rustls_pemfile::certs(&mut cert_file.as_slice())
            .map(|item| item.map_err(|_| anyhow!("Failed to parse certificate")))
            .collect::<Result<Vec<_>>>()?
    };

    if certs.is_empty() {
        anyhow::bail!("No certificates found in {cert_path}");
    }

    // Load the private key
    let mut private_keys = {
        let key_file = fs::read(key_path)?;
        rustls_pemfile::pkcs8_private_keys(&mut key_file.as_slice())
            .map(|item| item.map_err(|_| anyhow!("Failed to parse key")))
            .collect::<Result<Vec<_>>>()?
    };

    if private_keys.is_empty() {
        anyhow::bail!("No private keys found in {key_path}");
    }

    // Build the Rustls server configuration with client authentication
    let client_cert_verifier = Arc::new(WalletCertVerifier {
        wallet_cert: certs[0].as_ref().to_vec(),
    });

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_cert_verifier)
        .with_single_cert(certs, PrivateKeyDer::Pkcs8(private_keys.remove(0)))?;

    Ok(config)
}
