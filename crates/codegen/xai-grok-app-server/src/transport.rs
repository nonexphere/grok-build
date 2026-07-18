//! Transport boundary scaffold owned by App Server epics v1-03/v1-04.

use async_trait::async_trait;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    InProcess,
    Stdio,
    WebSocket,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionMeta {
    pub kind: TransportKind,
    pub authenticated: bool,
    pub remote: bool,
}

#[async_trait]
pub trait ProtocolConnection: Send {
    fn meta(&self) -> &ConnectionMeta;
    async fn receive(&mut self) -> Result<Option<String>, std::io::Error>;
    async fn send(&mut self, message: &str) -> Result<(), std::io::Error>;
    async fn close(&mut self) -> Result<(), std::io::Error>;
}
