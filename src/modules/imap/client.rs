use crate::modules::account::entity::Encryption;
use crate::modules::error::code::ErrorCode;
use crate::modules::error::RustMailerResult;
use crate::modules::imap::session::SessionStream;
use crate::modules::imap::stats::StatsWrapper;
use crate::modules::utils::net::establish_tcp_connection_with_timeout;
use crate::modules::utils::net::establish_tls_connection;
use crate::modules::utils::tls::establish_tls_stream;
use crate::raise_error;
use async_imap::Client as ImapClient;
use async_imap::Session as ImapSession;
use std::net::SocketAddr;
use std::net::ToSocketAddrs;
use std::ops::Deref;
use std::ops::DerefMut;
use tokio::io::BufWriter;
use tracing::debug;

#[derive(Debug)]
pub(crate) struct Client {
    inner: ImapClient<Box<dyn SessionStream>>,
}

impl Deref for Client {
    type Target = ImapClient<Box<dyn SessionStream>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Client {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

fn alpn(port: u16) -> &'static [&'static str] {
    if port == 993 {
        &[]
    } else {
        &["imap"]
    }
}

impl Client {
    fn new(stream: Box<dyn SessionStream>) -> Self {
        Self {
            inner: ImapClient::new(stream),
        }
    }

    pub(crate) async fn login(
        self,
        username: &str,
        password: &str,
    ) -> RustMailerResult<ImapSession<Box<dyn SessionStream>>> {
        let Client { inner, .. } = self;
        let session = inner.login(username, password).await.map_err(|(e, _)| {
            raise_error!(format!("{:#?}", e), ErrorCode::ImapAuthenticationFailed)
        })?;
        Ok(session)
    }

    pub(crate) async fn authenticate(
        self,
        authenticator: impl async_imap::Authenticator,
    ) -> RustMailerResult<ImapSession<Box<dyn SessionStream>>> {
        let Client { inner, .. } = self;
        let session = inner
            .authenticate("XOAUTH2", authenticator)
            .await
            .map_err(|(e, _)| {
                raise_error!(format!("{:#?}", e), ErrorCode::ImapAuthenticationFailed)
            })?;
        Ok(session)
    }

    pub async fn connection(
        domain: String,
        encryption: Encryption,
        port: u16,
        use_proxy: Option<u64>,
    ) -> RustMailerResult<Self> {
        let domain = &domain;
        let resolved_addr = Self::resolve_to_socket_addr(domain, port)?;
        debug!("Attempting IMAP connection to {domain} ({resolved_addr}).");
        match encryption {
            Encryption::Ssl => {
                Self::establish_secure_connection(resolved_addr, domain, use_proxy).await
            }
            Encryption::StartTls => {
                Self::establish_starttls_connection(resolved_addr, domain, use_proxy).await
            }
            Encryption::None => Self::establish_insecure_connection(resolved_addr, use_proxy).await,
        }
    }

    async fn establish_secure_connection(
        address: SocketAddr,
        server_hostname: &str,
        use_proxy: Option<u64>,
    ) -> RustMailerResult<Self> {
        // Establish the TLS connection with the specified parameters
        let tls_stream =
            establish_tls_connection(address, server_hostname, alpn(address.port()), use_proxy)
                .await?;
        let stats_stream = StatsWrapper::new(tls_stream);
        // Wrap the TLS stream in a buffered writer for efficient IO
        let buffered_stream = BufWriter::new(stats_stream);
        // Create a SessionStream trait object for further communication
        let session_stream = Box::new(buffered_stream);
        // Initialize the client with the session stream
        let mut client = Client::new(session_stream);
        // Read and validate the greeting response
        let _greeting = client
            .read_response()
            .await
            .ok_or_else(|| {
                raise_error!(
                    "failed to read greeting".into(),
                    ErrorCode::ImapCommandFailed
                )
            })?
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::ImapCommandFailed))?;

        // Return the established client
        Ok(client)
    }

    async fn establish_insecure_connection(
        address: SocketAddr,
        use_proxy: Option<u64>,
    ) -> RustMailerResult<Self> {
        // Establish the TCP connection without encryption
        let tcp_stream = establish_tcp_connection_with_timeout(address, use_proxy).await?;
        let stats_stream = StatsWrapper::new(tcp_stream);
        // Wrap the TCP stream in a buffered writer for efficient IO
        let buffered_stream = BufWriter::new(stats_stream);
        // Create a SessionStream trait object for further communication
        let session_stream: Box<dyn SessionStream> = Box::new(buffered_stream);
        // Initialize the client with the session stream
        let mut client = Client::new(session_stream);

        // Read and validate the greeting response
        let _greeting = client
            .read_response()
            .await
            .ok_or_else(|| {
                raise_error!(
                    "failed to read greeting".into(),
                    ErrorCode::ImapCommandFailed
                )
            })?
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::ImapCommandFailed))?;

        // Return the established client
        Ok(client)
    }

    async fn establish_starttls_connection(
        address: SocketAddr,
        server_hostname: &str,
        use_proxy: Option<u64>,
    ) -> RustMailerResult<Self> {
        // Establish the initial TCP connection
        let tcp_stream = establish_tcp_connection_with_timeout(address, use_proxy).await?;
        let stats_stream = StatsWrapper::new(tcp_stream);
        // Wrap the TCP stream in a buffered writer for efficient IO
        let buffered_tcp_stream = BufWriter::new(stats_stream);

        // Create a client for communication
        let mut client = async_imap::Client::new(buffered_tcp_stream);

        // Read and validate the greeting response
        let _greeting = client
            .read_response()
            .await
            .ok_or_else(|| {
                raise_error!(
                    "failed to read greeting".into(),
                    ErrorCode::ImapCommandFailed
                )
            })?
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::ImapCommandFailed))?;

        // Run the STARTTLS command to upgrade the connection to TLS
        client
            .run_command_and_check_ok("STARTTLS", None)
            .await
            .map_err(|_| {
                raise_error!(
                    "STARTTLS command failed".into(),
                    ErrorCode::ImapCommandFailed
                )
            })?;

        // Extract the TCP stream after running STARTTLS
        let buffered_tcp_stream = client.into_inner();
        let tcp_stream = buffered_tcp_stream.into_inner();
        // Wrap the TCP stream in TLS encryption
        let tls_stream = establish_tls_stream(server_hostname, &[], tcp_stream).await?;
        // Wrap the TLS stream in a buffered writer
        let buffered_stream = BufWriter::new(tls_stream);
        // Create a SessionStream trait object for further communication
        let session_stream: Box<dyn SessionStream> = Box::new(buffered_stream);
        // Initialize the client with the session stream
        let client = Client::new(session_stream);
        // Return the established client
        Ok(client)
    }

    fn resolve_to_socket_addr(domain: &str, port: u16) -> RustMailerResult<SocketAddr> {
        if domain.is_empty() || domain.contains(|c: char| !c.is_ascii() && c != '.') {
            return Err(raise_error!(
                "Invalid domain format".into(),
                ErrorCode::InvalidParameter
            ));
        }
        // Combine domain and port into a single address string
        let address = format!("{}:{}", domain, port);

        // Resolve the address into a SocketAddr
        let socket_addrs = address
            .to_socket_addrs()
            .map_err(|e| raise_error!(format!("{:#?}", e), ErrorCode::NetworkError))?;

        // Return the first valid SocketAddr
        socket_addrs.into_iter().next().ok_or_else(|| {
            raise_error!("Unable to resolve address".into(), ErrorCode::NetworkError)
        })
    }
}
