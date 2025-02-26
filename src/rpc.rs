use bitcoind::bitcoincore_rpc::{Auth, Client};
pub struct RPCConfig {
    /// The bitcoin node url
    pub url: String,
    /// The bitcoin node authentication mechanism
    pub auth: Auth,
    /// The wallet name in the bitcoin node, derive this from the descriptor.
    pub wallet_name: Option<String>,
}
const RPC_HOSTPORT: &str = "localhost:18443";

impl Default for RPCConfig {
    fn default() -> Self {
        Self {
            url: RPC_HOSTPORT.to_string(),
            auth: Auth::UserPass("regtestrpcuser".to_string(), "regtestrpcpass".to_string()),
            wallet_name: None,
        }
    }
}

#[derive(Debug)]
pub enum RPCError {
    Rpc(bitcoind::bitcoincore_rpc::Error),
}

impl From<bitcoind::bitcoincore_rpc::Error> for RPCError {
    fn from(value: bitcoind::bitcoincore_rpc::Error) -> Self {
        Self::Rpc(value)
    }
}
impl TryFrom<&RPCConfig> for Client {
    type Error = RPCError;

    fn try_from(config: &RPCConfig) -> Result<Self, RPCError> {
        let url = if let Some(wallet) = &config.wallet_name {
            format!("http://{}/wallet/{}", config.url, wallet)
        } else {
            format!("http://{}", config.url)
        };

        let rpc = Client::new(&url, config.auth.clone())?;
        Ok(rpc)
    }

}