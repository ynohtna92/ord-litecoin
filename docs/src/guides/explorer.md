Ordinal Explorer
================

The `ord` binary includes a block explorer. We host an instance of the block
explorer on mainnet at [ordinalslite.com](https://ordinalslite.com), on signet locally at
[localhost](http://localhost), and on testnet locally at
[localhost](http://localhost). As of version 0.16.0 the
wallet needs `ord server` running in the background. This is analogous to how
`litecoin-cli` needs `litecoind` running in the background.

### Running The Explorer
The server can be run locally with:

`ord server`

To specify a port add the `--http-port` flag:

`ord server --http-port 8080`

The JSON-API endpoints are enabled by default, to disable them add the
`--disable-json-api` flag (see [here](api.md) for more info):

`ord server --disable-json-api`

Search
------

The search box accepts a variety of object representations.

### Blocks

Blocks can be searched by hash, for example, the genesis block:

[000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f](https://ordinalslite.com/search/000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f)

### Transactions

Transactions can be searched by hash, for example, the genesis block coinbase
transaction:

[4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b](https://ordinalslite.com/search/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b)

### Outputs

Transaction outputs can be searched by outpoint, for example, the only output of
the genesis block coinbase transaction:

[4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0](https://ordinalslite.com/search/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0)

### Sats

Sats can be searched by integer, their position within the entire litecoin
supply:

[2099994106992659](https://ordinalslite.com/search/2099994106992659)

By decimal, their block and offset within that block:

[481824.0](https://ordinalslite.com/search/481824.0)

By degree, their cycle, blocks since the last halving, blocks since the last
difficulty adjustment, and offset within their block:

[1°0′0″0‴](https://ordinalslite.com/search/1°0′0″0‴)

By name, their base 26 representation using the letters "a" through "z":

[ahistorical](https://ordinalslite.com/search/ahistorical)

Or by percentile, the percentage of litecoin's supply that has been or will have
been issued when they are mined:

[100%](https://ordinalslite.com/search/100%)

```
