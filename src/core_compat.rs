use super::*;

/// Litecoin Core RPC versions use major * 10000 + minor * 100 + patch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CoreVersion(usize);

/// Decode the transparent block committed to by the header. Litecoin Core's
/// default RPC encoding appends an MWEB extension after the HogEx transaction;
/// confidential extension data is outside the ordinal index.
pub(crate) fn get_block(client: &Client, hash: &BlockHash) -> bitcoincore_rpc::Result<Block> {
  let hex: String = client.call("getblock", &[hash.to_string().into(), 0.into()])?;
  decode_block(&hex, hash).map_err(|error| bitcoincore_rpc::Error::ReturnedError(error.to_string()))
}

fn decode_block(hex: &str, hash: &BlockHash) -> Result<Block> {
  let bytes = hex::decode(hex)?;
  let mut cursor = bitcoin::io::Cursor::new(&bytes);
  let block = Block::consensus_decode(&mut cursor)?;
  ensure!(block.block_hash() == *hash, "RPC block hash mismatch");
  if cursor.position() != bytes.len() as u64 {
    ensure!(block.check_merkle_root(), "RPC block merkle root mismatch");
    let hogex = block.txdata.last().and_then(|tx| tx.output.first());
    ensure!(
      hogex.is_some_and(|output| {
        let script = output.script_pubkey.as_bytes();
        script.len() == 34 && script[..2] == [0x58, 0x20]
      }),
      "unexpected trailing data in RPC block"
    );
  }
  Ok(block)
}

impl CoreVersion {
  pub(crate) fn get(client: &Client) -> Result<Self> {
    let version = Self(client.version()?);
    version.require(210000, "ord wallets")?;
    Ok(version)
  }

  pub(crate) fn descriptors(self) -> bool {
    self.0 >= 240000
  }

  pub(crate) fn require(self, minimum: usize, feature: &str) -> Result {
    ensure!(
      self.0 >= minimum,
      "{feature} requires Litecoin Core {} or newer, current version is {}",
      Self(minimum),
      self,
    );
    Ok(())
  }
}

impl Display for CoreVersion {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}.{}.{}",
      self.0 / 10000,
      self.0 % 10000 / 100,
      self.0 % 100
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mweb_block_preserves_transparent_transaction_ids() {
    let hash = "1c2cd34bc199c7061cb9bb975f8a15ad8b4696def85c958cd9e6e2ce36d594de"
      .parse()
      .unwrap();
    let block = decode_block(
      include_str!("../tests/data/litecoin-mweb-block.hex").trim(),
      &hash,
    )
    .unwrap();
    assert_eq!(block.txdata.len(), 3);
    assert_eq!(
      block.txdata[2].compute_txid().to_string(),
      "e07612bbecf4ea5ec7e2c94965cdec8a79abd4684089d32d3462596fc3023652"
    );
  }

  #[test]
  fn truncated_mweb_block_is_rejected() {
    let hash = "1c2cd34bc199c7061cb9bb975f8a15ad8b4696def85c958cd9e6e2ce36d594de"
      .parse()
      .unwrap();
    assert!(decode_block(
      &include_str!("../tests/data/litecoin-mweb-block.hex")[..150],
      &hash
    )
    .is_err());
  }

  #[test]
  fn litecoin_genesis_blocks_match_core() {
    for (network, expected) in [
      (
        Network::Bitcoin,
        "12a765e31ffd4059bada1e25190f6e98c99d9714d334efa41a195a7e7e04bfe2",
      ),
      (
        Network::Testnet,
        "4966625a4b2851d9fdee139e56211a0d88575f59ed816ff5e6a63deb4e3e29a0",
      ),
      (
        Network::Signet,
        "5557c281d5f0f394bd38c9c4c6bd641d2920ffae34a3c7713da43eb2ce80a4aa",
      ),
      (
        Network::Regtest,
        "530827f38f93b43ed12af0b3ad25a288dc02ed74d6d7857862df51fc56c416f9",
      ),
    ] {
      assert_eq!(
        bitcoin::blockdata::constants::genesis_block(network)
          .block_hash()
          .to_string(),
        expected
      );
    }
  }

  #[test]
  fn descriptor_version_boundary() {
    for version in [210000, 210508, 239999] {
      assert!(!CoreVersion(version).descriptors());
      assert!(CoreVersion(version)
        .require(240000, "descriptor wallets")
        .is_err());
    }
    for version in [240000, 240001, 310000] {
      assert!(CoreVersion(version).descriptors());
      CoreVersion(version)
        .require(240000, "descriptor wallets")
        .unwrap();
    }
  }

  #[test]
  fn unsupported_features_report_required_and_actual_versions() {
    assert_eq!(
      CoreVersion(210508)
        .require(240000, "descriptor wallets")
        .unwrap_err()
        .to_string(),
      "descriptor wallets requires Litecoin Core 24.0.0 or newer, current version is 21.5.8"
    );
  }
}
