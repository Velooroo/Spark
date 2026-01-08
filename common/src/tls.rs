use anyhow::Result;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use std::io::Cursor;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::client::TlsStream as ClientTlsStream;
use tokio_rustls::server::TlsStream as ServerTlsStream;
use tokio_rustls::{TlsAcceptor, TlsConnector};
use tracing::info;

// ============================================================================
// TLS WRAPPER - Configurable Certificates
// ============================================================================

/// Connects to server with TLS (for CLI)
/// Verifies certificate only if NOT local network
pub async fn connect_tls(stream: TcpStream, host: &str) -> Result<ClientTlsStream<TcpStream>> {
    let is_local = is_local_network(host);

    let config = if is_local {
        info!("TLS without verification (local network)");
        rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(NoCertVerifier))
            .with_no_client_auth()
    } else {
        info!("TLS with full verification");
        rustls::ClientConfig::builder()
            .with_root_certificates(load_system_ca_roots())
            .with_no_client_auth()
    };

    let connector = TlsConnector::from(Arc::new(config));
    let domain = ServerName::try_from(host.to_string())?;

    let tls_stream = connector.connect(domain, stream).await?;
    Ok(tls_stream)
}

/// Accepts connection with TLS (for Daemon)
pub async fn accept_tls(stream: TcpStream) -> Result<ServerTlsStream<TcpStream>> {
    let (cert_pem, key_pem) = match load_custom_certs() {
        Some((c, k)) => {
            info!("Using custom TLS certificates");
            (c, k)
        }
        None => {
            info!("Generating self-signed certificate");
            generate_self_signed()?
        }
    };

    let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut Cursor::new(&cert_pem))
        .filter_map(|c| c.ok())
        .collect();

    if certs.is_empty() {
        anyhow::bail!("No certificates found");
    }

    let key = rustls_pemfile::private_key(&mut Cursor::new(&key_pem))?
        .ok_or_else(|| anyhow::anyhow!("No private key found"))?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let tls_stream = acceptor.accept(stream).await?;

    Ok(tls_stream)
}

// ============================================================================
// HELPERS
// ============================================================================

fn is_local_network(host: &str) -> bool {
    host.starts_with("127.")
        || host.starts_with("192.168.")
        || host.starts_with("10.")
        || host.starts_with("172.16.")
        || host.starts_with("172.17.")
        || host.starts_with("172.18.")
        || host.starts_with("172.19.")
        || host.starts_with("172.20.")
        || host.starts_with("172.21.")
        || host.starts_with("172.22.")
        || host.starts_with("172.23.")
        || host.starts_with("172.24.")
        || host.starts_with("172.25.")
        || host.starts_with("172.26.")
        || host.starts_with("172.27.")
        || host.starts_with("172.28.")
        || host.starts_with("172.29.")
        || host.starts_with("172.30.")
        || host.starts_with("172.31.")
        || host == "localhost"
}

fn load_system_ca_roots() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();

    // Load built-in Mozilla CA database (includes Let's Encrypt)
    roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    roots
}

fn load_custom_certs() -> Option<(Vec<u8>, Vec<u8>)> {
    if let (Ok(cert), Ok(key)) = (
        std::env::var("SPARK_TLS_CERT"),
        std::env::var("SPARK_TLS_KEY"),
    ) {
        return Some((cert.into_bytes(), key.into_bytes()));
    }

    if let (Ok(cert_path), Ok(key_path)) = (
        std::env::var("SPARK_TLS_CERT_FILE"),
        std::env::var("SPARK_TLS_KEY_FILE"),
    ) {
        if let (Ok(cert), Ok(key)) = (std::fs::read(&cert_path), std::fs::read(&key_path)) {
            return Some((cert, key));
        }
    }

    if let (Ok(cert), Ok(key)) = (
        std::fs::read("/etc/letsencrypt/live/sparkle/fullchain.pem"),
        std::fs::read("/etc/letsencrypt/live/sparkle/privkey.pem"),
    ) {
        return Some((cert, key));
    }

    None
}

fn generate_self_signed() -> Result<(Vec<u8>, Vec<u8>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    Ok((
        cert.serialize_pem()?.into_bytes(),
        cert.serialize_private_key_pem().into_bytes(),
    ))
}

// ============================================================================
// VERIFIER
// ============================================================================

#[derive(Debug)]
struct NoCertVerifier;

impl rustls::client::danger::ServerCertVerifier for NoCertVerifier {
    fn verify_server_cert(
        &self,
        _: &CertificateDer,
        _: &[CertificateDer],
        _: &ServerName,
        _: &[u8],
        _: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer,
        _: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ED25519,
        ]
    }
}
