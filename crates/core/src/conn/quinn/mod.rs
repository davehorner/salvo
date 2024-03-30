//! `QuinnListener`` and utils.
use std::io::{Error as IoError, Result as IoResult};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::vec;

use futures_util::Stream;
use salvo_http3::http3_quinn;
pub use salvo_http3::http3_quinn::ServerConfig;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio_util::sync::CancellationToken;

use crate::conn::rustls::RustlsConfig;
use crate::conn::{HttpBuilder, IntoConfigStream};
use crate::fuse::ArcFusewire;
use crate::http::HttpConnection;
use crate::service::HyperHandler;

mod builder;
pub use builder::Builder;
mod listener;
pub use listener::{QuinnAcceptor, QuinnListener};

impl TryInto<ServerConfig> for RustlsConfig {
    type Error = IoError;
    fn try_into(self) -> IoResult<ServerConfig> {
        let mut crypto = self.build_server_config_old()?;
        crypto.alpn_protocols = vec![b"h3-29".to_vec(), b"h3-28".to_vec(), b"h3-27".to_vec(), b"h3".to_vec()];
        Ok(ServerConfig::with_crypto(Arc::new(crypto)))
    }
}

/// Http3 Connection.
pub struct H3Connection {
    inner: http3_quinn::Connection,
    fusewire: Option<ArcFusewire>,
}
impl H3Connection {
    pub(crate) fn new(inner: http3_quinn::Connection, fusewire: Option<ArcFusewire>) -> Self {
        Self { inner, fusewire }
    }
    /// Get inner quinn connection.
    pub fn into_inner(self) -> http3_quinn::Connection {
        self.inner
    }
}
impl Deref for H3Connection {
    type Target = http3_quinn::Connection;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for H3Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl AsyncRead for H3Connection {
    fn poll_read(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &mut ReadBuf<'_>) -> Poll<IoResult<()>> {
        unimplemented!()
    }
}

impl AsyncWrite for H3Connection {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &[u8]) -> Poll<IoResult<usize>> {
        unimplemented!()
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        unimplemented!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<IoResult<()>> {
        unimplemented!()
    }
}

impl HttpConnection for H3Connection {
    async fn serve(
        self,
        handler: HyperHandler,
        builder: Arc<HttpBuilder>,
        graceful_stop_token: CancellationToken,
    ) -> IoResult<()> {
        builder.quinn.serve_connection(self, handler, graceful_stop_token).await
    }
    fn fusewire(&self) -> Option<ArcFusewire> {
        self.fusewire.clone()
    }
}

impl<T> IntoConfigStream<ServerConfig> for T
where
    T: Stream<Item = ServerConfig> + Send + 'static,
{
    type Stream = T;

    fn into_stream(self) -> Self {
        self
    }
}
