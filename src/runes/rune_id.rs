use {super::*, std::num::TryFromIntError};

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, Ord, PartialOrd, Default)]
pub struct RuneId {
  pub block: u32,
  pub tx: u16,
}

impl TryFrom<u128> for RuneId {
  type Error = TryFromIntError;

  fn try_from(n: u128) -> Result<Self, Self::Error> {
    Ok(Self {
      block: u32::try_from(n >> 16)?,
      tx: u16::try_from(n & 0xFFFF).unwrap(),
    })
  }
}

impl From<RuneId> for u128 {
  fn from(id: RuneId) -> Self {
    u128::from(id.block) << 16 | u128::from(id.tx)
  }
}

impl Display for RuneId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}:{}", self.block, self.tx,)
  }
}

impl FromStr for RuneId {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (height, index) = s
      .split_once(':')
      .ok_or_else(|| anyhow!("invalid rune ID: {s}"))?;

    Ok(Self {
      block: height.parse()?,
      tx: index.parse()?,
    })
  }
}

impl Serialize for RuneId {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for RuneId {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    DeserializeFromStr::with(deserializer)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rune_id_to_128() {
    assert_eq!(
      0b11_0000_0000_0000_0001u128,
      RuneId { block: 3, tx: 1 }.into()
    );
  }

  #[test]
  fn display() {
    assert_eq!(RuneId { block: 1, tx: 2 }.to_string(), "1:2");
  }

  #[test]
  fn from_str() {
    assert!(":".parse::<RuneId>().is_err());
    assert!("1:".parse::<RuneId>().is_err());
    assert!(":2".parse::<RuneId>().is_err());
    assert!("a:2".parse::<RuneId>().is_err());
    assert!("1:a".parse::<RuneId>().is_err());
    assert_eq!("1:2".parse::<RuneId>().unwrap(), RuneId { block: 1, tx: 2 });
  }

  #[test]
  fn try_from() {
    assert_eq!(
      RuneId::try_from(0x060504030201).unwrap(),
      RuneId {
        block: 0x06050403,
        tx: 0x0201
      }
    );

    assert!(RuneId::try_from(0x07060504030201).is_err());
  }

  #[test]
  fn serde() {
    let rune_id = RuneId { block: 1, tx: 2 };
    let json = "\"1:2\"";
    assert_eq!(serde_json::to_string(&rune_id).unwrap(), json);
    assert_eq!(serde_json::from_str::<RuneId>(json).unwrap(), rune_id);
  }
}
