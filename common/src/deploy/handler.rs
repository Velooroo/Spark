use crate::deploy::gateway::SharedGatewayState;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::error;

pub async fn handle_deploy_request<S>(_socket: S, _gateway: SharedGatewayState, _config: &crate::config::CommandConfig)
where
    S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
{
    // This is a stub - actual implementation is in daemon
    error!("handle_deploy_request called from common - should be in daemon");
}