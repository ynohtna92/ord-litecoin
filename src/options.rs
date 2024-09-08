use super::*;

#[derive(Clone, Default, Debug, Parser)]
#[command(group(
  ArgGroup::new("chains")
    .required(false)
    .args(&["chain_argument", "signet", "regtest", "testnet"]),
))]
pub struct Options {
  #[arg(long, help = "Load Litecoin Core data dir from <LITECOIN_DATA_DIR>.")]
  pub(crate) litecoin_data_dir: Option<PathBuf>,
  #[arg(
    long,
    help = "Authenticate to Litecoin Core RPC with <LITECOIN_RPC_PASSWORD>."
  )]
  pub(crate) litecoin_rpc_password: Option<String>,
  #[arg(long, help = "Connect to Litecoin Core RPC at <LITECOIN_RPC_URL>.")]
  pub(crate) litecoin_rpc_url: Option<String>,
  #[arg(
    long,
    help = "Authenticate to Litecoin Core RPC as <LITECOIN_RPC_USERNAME>."
  )]
  pub(crate) litecoin_rpc_username: Option<String>,
  #[arg(long, help = "Max <N> requests in flight. [default: 12]")]
  pub(crate) litecoin_rpc_limit: Option<u32>,
  #[arg(long = "chain", value_enum, help = "Use <CHAIN>. [default: mainnet]")]
  pub(crate) chain_argument: Option<Chain>,
  #[arg(
    long,
    help = "Commit to index every <COMMIT_INTERVAL> blocks. [default: 5000]"
  )]
  pub(crate) commit_interval: Option<usize>,
  #[arg(long, help = "Load configuration from <CONFIG>.")]
  pub(crate) config: Option<PathBuf>,
  #[arg(long, help = "Load configuration from <CONFIG_DIR>.")]
  pub(crate) config_dir: Option<PathBuf>,
  #[arg(long, help = "Load Litecoin Core RPC cookie file from <COOKIE_FILE>.")]
  pub(crate) cookie_file: Option<PathBuf>,
  #[arg(long, alias = "datadir", help = "Store index in <DATA_DIR>.")]
  pub(crate) data_dir: Option<PathBuf>,
  #[arg(
    long,
    help = "Don't look for inscriptions below <FIRST_INSCRIPTION_HEIGHT>."
  )]
  pub(crate) first_inscription_height: Option<u32>,
  #[arg(long, help = "Limit index to <HEIGHT_LIMIT> blocks.")]
  pub(crate) height_limit: Option<u32>,
  #[arg(long, help = "Use index at <INDEX>.")]
  pub(crate) index: Option<PathBuf>,
  #[arg(long, help = "Track unspent output addresses.")]
  pub(crate) index_addresses: bool,
  #[arg(
    long,
    help = "Set index cache size to <INDEX_CACHE_SIZE> bytes. [default: 1/4 available RAM]"
  )]
  pub(crate) index_cache_size: Option<usize>,
  #[arg(
    long,
    help = "Track location of runes. RUNES ARE IN AN UNFINISHED PRE-ALPHA STATE AND SUBJECT TO CHANGE AT ANY TIME."
  )]
  pub(crate) index_runes: bool,
  #[arg(long, help = "Track location of all satoshis.")]
  pub(crate) index_sats: bool,
  #[arg(long, help = "Store transactions in index.")]
  pub(crate) index_transactions: bool,
  #[arg(long, help = "Run in integration test mode.")]
  pub(crate) integration_test: bool,
  #[clap(long, short, long, help = "Specify output format. [default: json]")]
  pub(crate) format: Option<OutputFormat>,
  #[arg(
    long,
    short,
    alias = "noindex_inscriptions",
    help = "Do not index inscriptions."
  )]
  pub(crate) no_index_inscriptions: bool,
  #[arg(
    long,
    help = "Require basic HTTP authentication with <SERVER_PASSWORD>. Credentials are sent in cleartext. Consider using authentication in conjunction with HTTPS."
  )]
  pub(crate) server_password: Option<String>,
  #[arg(
    long,
    help = "Require basic HTTP authentication with <SERVER_USERNAME>. Credentials are sent in cleartext. Consider using authentication in conjunction with HTTPS."
  )]
  pub(crate) server_username: Option<String>,
  #[arg(long, short, help = "Use regtest. Equivalent to `--chain regtest`.")]
  pub(crate) regtest: bool,
  #[arg(long, short, help = "Use signet. Equivalent to `--chain signet`.")]
  pub(crate) signet: bool,
  #[arg(long, short, help = "Use testnet. Equivalent to `--chain testnet`.")]
  pub(crate) testnet: bool,
}
