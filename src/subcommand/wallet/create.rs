use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub mnemonic: Mnemonic,
  pub passphrase: Option<String>,
  pub message: String,
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
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let mut entropy = [0; 16];
    rand::thread_rng().fill_bytes(&mut entropy);

    let mnemonic = Mnemonic::from_entropy(&entropy)?;

    initialize_wallet(&options, mnemonic.to_seed(self.passphrase.clone()))?;

    let mut warn = String::new();
    if !self.passphrase.is_empty() {
      warn += "Passphrase is not used in wallet creation as descriptor wallets are not supported.";
    }
    warn += "Ord wallet created! The mnemonic above is not used as descriptor wallets are not \
      supported in Litecoincore!!!! Please make a backup of the \
      wallet.dat file and store it in a safe place.";

    Ok(Box::new(Output {
      mnemonic,
      passphrase: Some(self.passphrase),
      message: warn,
    }))
  }
}
