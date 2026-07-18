//! Processor boundary only. Implementation belongs to App Server epics v1-03+
//! and must remain a dispatcher over `GrokRuntimeFacade`, never a second runtime.

pub mod transport;

use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessorError {
    pub code: i64,
    pub message: String,
}

#[async_trait]
pub trait AppServerProcessor: Send + Sync {
    async fn process(&self, method: &str, params: Value) -> Result<Value, ProcessorError>;
}
