mod rpc;
mod util;
use crate::util::parse_proxy_auth;
use clap::{Parser, Subcommand};
use bitcoind::bitcoincore_rpc::{Auth, Client};
use dustbuster::DustBuster;
use crate::rpc::{RPCConfig, RPCError};

#[derive(Parser)]
#[command(name = "dust-spender")]
#[command(about = "CLI tool to spend dust UTXOs to fees", long_about = None)]
struct Cli {
    #[clap(
        name = "RPC_URL",
        long,
        short = 'r',
        default_value = "127.0.0.1:48332"
    )]
    pub rpc: String,
    #[clap(
        name = "AUTH",
        short = 'a',
        long,
        value_parser = parse_proxy_auth,
        default_value = "user:password",
    )]
    pub auth: (String, String),
    #[clap(name = "WALLET", long, short = 'w')]
    pub(crate) wallet_name: Option<String>,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List dust UTXOs
    ListDust {
        /// Min Relay Fee rate in sat/vB
        #[arg(short, long, default_value_t = 1)]
        min_relay_fee: u64,
        /// Bitcoin address to filter utxos by
        #[arg(short, long)]
        address: Option<String>,
    },
    /// Create an unsigned PSBT spending dust utxos to fees
    CreatePsbt {
        /// Min Relay Fee rate in sat/vB
        #[arg(short, long, default_value_t = 1)]
        min_relay_fee: u64,
        /// Bitcoin address to filter utxos by
        #[arg(short, long)]
        address: String,
        /// Number of utxos to be included
        #[arg(short, long, default_value_t = 100)]
        utxo_count: u64,
    },
}

fn main() -> Result<(), RPCError> {
    let args = Cli::parse();

    let mut rpc_config = RPCConfig {
        url: args.rpc,
        auth: Auth::UserPass(args.auth.0, args.auth.1),
        wallet_name: args.wallet_name,
    };
    rpc_config = Some(rpc_config).unwrap_or_default();
    let rpc = Client::try_from(&rpc_config)?;
    
    let dust_buster = DustBuster::new(rpc);

    let _ = match &args.command {
        Commands::ListDust { min_relay_fee, address } => {
            dust_buster.list_dust(*min_relay_fee, address)
        },
        Commands::CreatePsbt { min_relay_fee, address, utxo_count } => dust_buster.create_psbt(*min_relay_fee, address.to_string(), *utxo_count),
    };
    Ok(())
}

