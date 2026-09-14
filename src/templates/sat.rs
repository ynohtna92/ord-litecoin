use super::*;

#[derive(Boilerplate)]
pub(crate) struct SatHtml {
  pub(crate) address: Option<Address>,
  pub(crate) block: Option<BlockHeader>,
  pub(crate) blocktime: Blocktime,
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) sat: Sat,
  pub(crate) satpoint: Option<SatPoint>,
}

impl SatHtml {
  fn luck_odds(luck: u8) -> String {
    match 1u128.checked_shl(luck.into()) {
      Some(odds) => format!("1 in {odds}"),
      None => format!("1 in 2^{luck}"),
    }
  }
}

impl PageContent for SatHtml {
  fn title(&self) -> String {
    format!("Sat {}", self.sat)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn luck_odds() {
    assert_eq!(SatHtml::luck_odds(0), "1 in 1");
    assert_eq!(SatHtml::luck_odds(11), "1 in 2048");
    assert_eq!(
      SatHtml::luck_odds(127),
      "1 in 170141183460469231731687303715884105728",
    );
    assert_eq!(SatHtml::luck_odds(128), "1 in 2^128");
  }

  #[test]
  fn first() {
    assert_regex_match!(
      SatHtml {
        address: None,
        block: None,
        sat: Sat(0),
        satpoint: None,
        blocktime: Blocktime::confirmed(0),
        inscriptions: Vec::new(),
      },
      "
        <h1>Sat 0</h1>
        <dl>
          <dt>decimal</dt><dd>0.0</dd>
          <dt>degree</dt><dd>0°0′0″0‴</dd>
          <dt>percentile</dt><dd>0%</dd>
          <dt>name</dt><dd>bgmbqkqiqsxl</dd>
          <dt>cycle</dt><dd>0</dd>
          <dt>epoch</dt><dd>0</dd>
          <dt>period</dt><dd>0</dd>
          <dt>block</dt><dd><a href=/block/0>0</a></dd>
          <dt>offset</dt><dd>0</dd>
          <dt>timestamp</dt><dd><time>1970-01-01 00:00:00 UTC</time></dd>
          <dt>rarity</dt><dd><span class=mythic>mythic</span></dd>
          <dt>charms</dt>
          <dd>
            <span title=coin>🪙</span>
            <span title=mythic>🎃</span>
            <span title=palindrome>🦋</span>
          </dd>
        </dl>
        .*
        prev
        <a class=next href=/sat/1>next</a>
        .*
      "
      .unindent()
    );
  }

  #[test]
  fn last() {
    assert_regex_match!(
      SatHtml {
        address: None,
        block: None,
        sat: Sat(8399999990759999),
        satpoint: None,
        blocktime: Blocktime::confirmed(0),
        inscriptions: Vec::new(),
      },
      "
        <h1>Sat 8399999990759999</h1>
        <dl>
          <dt>decimal</dt><dd>27719999.0</dd>
          <dt>degree</dt><dd>10°839999′2015″0‴</dd>
          <dt>percentile</dt><dd>100%</dd>
          <dt>name</dt><dd>a</dd>
          <dt>cycle</dt><dd>10</dd>
          <dt>epoch</dt><dd>32</dd>
          <dt>period</dt><dd>13749</dd>
          <dt>block</dt><dd><a href=/block/27719999>27719999</a></dd>
          <dt>offset</dt><dd>0</dd>
          <dt>timestamp</dt><dd><time>1970-01-01 00:00:00 UTC</time></dd>
          <dt>rarity</dt><dd><span class=uncommon>uncommon</span></dd>
          <dt>charms</dt>
          <dd>
            <span title=uncommon>🌱</span>
          </dd>
        </dl>
        .*
        <a class=prev href=/sat/8399999990759998>prev</a>
        next
        .*
      "
      .unindent()
    );
  }

  #[test]
  fn sat_with_next_and_prev() {
    assert_regex_match!(
      SatHtml {
        address: None,
        block: None,
        sat: Sat(1),
        satpoint: None,
        blocktime: Blocktime::confirmed(0),
        inscriptions: Vec::new(),
      },
      r"<h1>Sat 1</h1>.*<a class=prev href=/sat/0>prev</a>\n<a class=next href=/sat/2>next</a>.*",
    );
  }

  #[test]
  fn sat_with_inscription() {
    assert_regex_match!(
      SatHtml {
        address: None,
        block: None,
        sat: Sat(0),
        satpoint: None,
        blocktime: Blocktime::confirmed(0),
        inscriptions: vec![inscription_id(1)],
      },
      "
        <h1>Sat 0</h1>
        .*
          <dt>inscriptions</dt>
          <dd class=thumbnails>
            <a href=/inscription/1{64}i1>.*</a>
          </dd>
        .*"
        .unindent(),
    );
  }

  #[test]
  fn sat_with_reinscription() {
    assert_regex_match!(
      SatHtml {
        address: None,
        block: None,
        sat: Sat(0),
        satpoint: None,
        blocktime: Blocktime::confirmed(0),
        inscriptions: vec![inscription_id(1), inscription_id(2)],
      },
      "
        <h1>Sat 0</h1>
        .*
          <dt>inscriptions</dt>
          <dd class=thumbnails>
            <a href=/inscription/1{64}i1>.*</a>
            <a href=/inscription/2{64}i2>.*</a>
          </dd>
        .*"
        .unindent(),
    );
  }

  #[test]
  fn last_sat_next_link_is_disabled() {
    assert_regex_match!(
      SatHtml {
        address: None,
        block: None,
        sat: Sat::LAST,
        satpoint: None,
        blocktime: Blocktime::confirmed(0),
        inscriptions: Vec::new(),
      },
      r"<h1>Sat 8399999990759999</h1>.*<a class=prev href=/sat/8399999990759998>prev</a>\nnext.*",
    );
  }

  #[test]
  fn sat_with_satpoint() {
    assert_regex_match!(
      SatHtml {
        address: None,
        block: None,
        sat: Sat(0),
        satpoint: Some(satpoint(1, 0)),
        blocktime: Blocktime::confirmed(0),
        inscriptions: Vec::new(),
      },
      "<h1>Sat 0</h1>.*<dt>location</dt><dd><a class=collapse href=/satpoint/1{64}:1:0>1{64}:1:0</a></dd>.*",
    );
  }

  #[test]
  fn sat_with_luck() {
    assert_regex_match!(
      SatHtml {
        address: None,
        block: Some(bitcoin::constants::genesis_block(Network::Bitcoin).header),
        sat: Sat(0),
        satpoint: None,
        blocktime: Blocktime::confirmed(0),
        inscriptions: Vec::new(),
      },
      "<h1>Sat 0</h1>.*<dt>luck</dt><dd><span title=\"1 in 2\">1</span></dd>.*",
    );
  }

  #[test]
  fn sat_with_address() {
    assert_regex_match!(
      SatHtml {
        address: Some(address(0)),
        block: None,
        sat: Sat(0),
        satpoint: Some(satpoint(1, 0)),
        blocktime: Blocktime::confirmed(0),
        inscriptions: Vec::new(),
      },
      "<h1>Sat 0</h1>.*<dt>address</dt><dd class=monospace><a href=/address/ltc1qw508d6qejxtdg4y5r3zarvary0c5xw7kgmn4n9>ltc1qw508d6qejxtdg4y5r3zarvary0c5xw7kgmn4n9</a></dd>.*",
    );
  }
}
