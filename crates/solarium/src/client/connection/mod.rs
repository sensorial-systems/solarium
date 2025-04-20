use solana_sdk::nonblocking::rpc_client::RpcClient;

#[derive(Debug, Clone)]
pub struct Connection {
    client: std::sync::Arc<RpcClient>,
}

impl Connection {
    pub fn new(client: solana_sdk::nonblocking::rpc_client::RpcClient) -> Self {
        Self { client }
    }
}
