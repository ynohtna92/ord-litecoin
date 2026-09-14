use {
  super::*,
  bitcoin::secp256k1::rand::{self, RngCore},
};

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub mnemonic: Mnemonic,
  pub passphrase: Option<String>,
}

#[derive(Debug, Parser)]
pub(crate) struct Create {
  #[arg(
    long,
    default_value = "",
    help = "Use <PASSPHRASE> to derive wallet seed."
  )]
  pub(crate) passphrase: String,
}

impl Create {
  pub(crate) fn run(self, name: String, settings: &Settings) -> SubcommandResult {
    let client = Wallet::check_version(settings.bitcoin_rpc_client(None)?)?;
    if !crate::core_compat::CoreVersion::get(&client)?.descriptors() {
      ensure!(
        self.passphrase.is_empty(),
        "mnemonic passphrases require Litecoin Core 24.0.0 or newer"
      );
      client.create_wallet(&name, None, Some(false), None, None)?;
      #[derive(Serialize)]
      struct LegacyOutput {
        wallet: String,
        backup: &'static str,
      }
      return Ok(Some(Box::new(LegacyOutput {
        wallet: name,
        backup: "Back up wallet.dat with Litecoin Core. This legacy wallet has no mnemonic backup.",
      })));
    }

    let mut entropy = [0; 16];
    rand::thread_rng().fill_bytes(&mut entropy);

    let mnemonic = Mnemonic::from_entropy(&entropy)?;

    Wallet::initialize(
      name,
      settings,
      mnemonic.to_seed(&self.passphrase),
      bitcoincore_rpc::json::Timestamp::Now,
    )?;

    Ok(Some(Box::new(Output {
      mnemonic,
      passphrase: Some(self.passphrase),
    })))
  }
}
