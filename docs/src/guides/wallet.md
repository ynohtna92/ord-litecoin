Wallet
======

Individual sats can be inscribed with arbitrary content, creating
Litecoin-native digital artifacts that can be held in a Litecoin wallet and
transferred using Litecoin transactions. Inscriptions are as durable, immutable,
secure, and decentralized as Litecoin itself.

Working with inscriptions requires a Litecoin full node, to give you a view of
the current state of the Litecoin blockchain, and a wallet that can create
inscriptions and perform sat control when constructing transactions to send
inscriptions to another wallet.

Litecoin Core provides both a Litecoin full node and wallet. However, the Litecoin
Core wallet cannot create inscriptions and does not perform sat control.

This requires [`ord`](https://github.com/ynohtna92/ord-litecoin), the ordinal utility. `ord`
doesn't implement its own wallet, so `ord wallet` subcommands interact with
Litecoin Core wallets.

This guide covers:

1. Installing Litecoin Core
2. Syncing the Litecoin blockchain
3. Creating a Litecoin Core wallet
4. Using `ord wallet receive` to receive sats
5. Creating inscriptions with `ord wallet inscribe`
6. Sending inscriptions with `ord wallet send`
7. Receiving inscriptions with `ord wallet receive`
8. Batch inscribing with `ord wallet inscribe --batch`

Getting Help
------------

If you get stuck, try asking for help on the [Ordinals Discord
Server](https://discord.com/invite/87cjuz4FYg), or checking GitHub for relevant
[issues](https://github.com/ynohtna92/ord-litecoin/issues) and
[discussions](https://github.com/ynohtna92/ord-litecoin/discussions).

Installing Litecoin Core
-----------------------

Litecoin Core is available from [litecoin.org](https://litecoin.org/) on
the [download page](https://litecoin.org/).

Making inscriptions requires Litecoin Core 21.5.8; Taproot wallets require a compatible Core 24 development build.

This guide does not cover installing Litecoin Core in detail. Once Litecoin Core
is installed, you should be able to run `litecoind -version` successfully from
the command line. Do *NOT* use `litecoin-qt`.

Configuring Litecoin Core
------------------------

`ord` requires Litecoin Core's transaction index and rest interface.

To configure your Litecoin Core node to maintain a transaction
index, add the following to your `litecoin.conf`:

```
txindex=1
```

Or, run `litecoind` with `-txindex`:

```
litecoind -txindex
```

Details on creating or modifying your `litecoin.conf` file can be found
[here](https://github.com/litecoin-project/litecoin/blob/master/doc/bitcoin-conf.md).

Syncing the Litecoin Blockchain
------------------------------

To sync the chain, run:

```
litecoind -txindex
```

…and leave it running until `getblockcount`:

```
litecoin-cli getblockcount
```

agrees with the block count on a block explorer like [the litecoinspace.org block
explorer](https://litecoinspace.org/). `ord` interacts with `litecoind`, so you
should leave `litecoind` running in the background when you're using `ord`.

The blockchain takes about 600GB of disk space. If you have an external drive
you want to store blocks on, use the configuration option
`blocksdir=<external_drive_path>`. This is much simpler than using the
`datadir` option because the cookie file will still be in the default location
for `litecoin-cli` and `ord` to find.

Troubleshooting
---------------

Make sure you can access `litecoind` with `litecoin-cli -getinfo` and that it is
fully synced.

If `litecoin-cli -getinfo` returns `Could not connect to the server`, `litecoind`
is not running.

Make sure `rpcuser`, `rpcpassword`, or `rpcauth` are *NOT* set in your
`litecoin.conf` file. `ord` requires using cookie authentication. Make sure there
is a file `.cookie` in your litecoin data directory.

If `litecoin-cli -getinfo` returns `Could not locate RPC credentials`, then you
must specify the cookie file location.
If you are using a custom data directory (specifying the `datadir` option),
then you must specify the cookie location like
`litecoin-cli -rpccookiefile=<your_bitcoin_datadir>/.cookie -getinfo`.
When running `ord` you must specify the cookie file location with
`--cookie-file=<your_bitcoin_datadir>/.cookie`.

Make sure you do *NOT* have `disablewallet=1` in your `litecoin.conf` file. If
`litecoin-cli listwallets` returns `Method not found` then the wallet is disabled
and you won't be able to use `ord`.

Make sure `txindex=1` is set. Run `litecoin-cli getindexinfo` and it should
return something like
```json
{
  "txindex": {
    "synced": true,
    "best_block_height": 776546
  }
}
```
If it only returns `{}`, `txindex` is not set.
If it returns `"synced": false`, `litecoind` is still creating the `txindex`.
Wait until `"synced": true` before using `ord`.

If you have `maxuploadtarget` set it can interfere with fetching blocks for
`ord` index. Either remove it or set `whitebind=127.0.0.1:9333`.

Installing `ord`
----------------

The `ord` utility is written in Rust and can be built from
[source](https://github.com/ynohtna92/ord-litecoin). Pre-built binaries are available on the
[releases page](https://github.com/ynohtna92/ord-litecoin/releases).

You can install the latest pre-built binary from the command line with:

```sh
curl --proto '=https' --tlsv1.2 -fsLS https://raw.githubusercontent.com/ynohtna92/ord-litecoin/master/install.sh | bash -s
```

Once `ord` is installed, you should be able to run:

```
ord --version
```

Which prints out `ord`'s version number.

Creating a Wallet
-----------------

`ord` uses `litecoind` to manage private keys, sign transactions, and
broadcast transactions to the Litecoin network. Additionally the `ord wallet`
requires [`ord server`](explorer.md) running in the background. Make sure these
programs are running:

```
litecoind -txindex
```

```
ord server
```

To create a wallet named `ord`, the default, for use with `ord wallet`, run:

```
ord wallet create
```

This will print out your seed phrase mnemonic, store it somewhere safe.

```
{
  "mnemonic": "dignity buddy actor toast talk crisp city annual tourist orient similar federal",
  "passphrase": ""
}
```

If you want to specify a different name or use an `ord server` running on a
non-default URL you can set these options:

```
ord wallet --name foo --server-url http://127.0.0.1:8080 create
```

To see all available wallet options you can run:

```
ord wallet help
```

Restoring and Dumping Wallet
----------------------------

The `ord` wallet uses descriptors, so you can export the output descriptors and
import them into another descriptor-based wallet. To export the wallet
descriptors, which include your private keys:

```
$ ord wallet dump
==========================================
= THIS STRING CONTAINS YOUR PRIVATE KEYS =
=        DO NOT SHARE WITH ANYONE        =
==========================================
{
  "wallet_name": "ord",
  "descriptors": [
    {
      "desc": "tr([551ac972/86'/1'/0']tprv8h4xBhrfZwX9o1XtUMmz92yNiGRYjF9B1vkvQ858aN1UQcACZNqN9nFzj3vrYPa4jdPMfw4ooMuNBfR4gcYm7LmhKZNTaF4etbN29Tj7UcH/0/*)#uxn94yt5",
      "timestamp": 1296688602,
      "active": true,
      "internal": false,
      "range": [
        0,
        999
      ],
      "next": 0
    },
    {
      "desc": "tr([551ac972/86'/1'/0']tprv8h4xBhrfZwX9o1XtUMmz92yNiGRYjF9B1vkvQ858aN1UQcACZNqN9nFzj3vrYPa4jdPMfw4ooMuNBfR4gcYm7LmhKZNTaF4etbN29Tj7UcH/1/*)#djkyg3mv",
      "timestamp": 1296688602,
      "active": true,
      "internal": true,
      "range": [
        0,
        999
      ],
      "next": 0
    }
  ]
}
```

An `ord` wallet can be restored from a mnemonic:

```
ord wallet restore --from mnemonic
```

Type your mnemonic and press return.

To restore from a descriptor in `descriptor.json`:

```
cat descriptor.json | ord wallet restore --from descriptor
```

To restore from a descriptor in the clipboard:

```
ord wallet restore --from descriptor
```

Paste the descriptor into the terminal and press CTRL-D on unix and CTRL-Z
on Windows.

Receiving Sats
--------------

Inscriptions are made on individual sats, using normal Litecoin transactions
that pay fees in sats, so your wallet will need some sats.

Get a new address from your `ord` wallet by running:

```
ord wallet receive
```

And send it some funds.

You can see pending transactions with:

```
ord wallet transactions
```

Once the transaction confirms, you should be able to see the transactions
outputs with `ord wallet outputs`.

Creating Inscription Content
----------------------------

Sats can be inscribed with any kind of content, but the `ord` wallet only
supports content types that can be displayed by the `ord` block explorer.

Additionally, inscriptions are included in transactions, so the larger the
content, the higher the fee that the inscription transaction must pay.

Inscription content is included in transaction witnesses, which receive the
witness discount. To calculate the approximate fee that an inscribe transaction
will pay, divide the content size by four and multiply by the fee rate.

Inscription transactions must be less than 400,000 weight units, or they will
not be relayed by Litecoin Core. One byte of inscription content costs one
weight unit. Since an inscription transaction includes not just the inscription
content, limit inscription content to less than 400,000 weight units. 390,000
weight units should be safe.

Creating Inscriptions
---------------------

To create an inscription with the contents of `FILE`, run:

```
ord wallet inscribe --fee-rate FEE_RATE --file FILE
```

Ord will output two transactions IDs, one for the commit transaction, and one
for the reveal transaction, and the inscription ID. Inscription IDs are of the
form `TXIDiN`, where `TXID` is the transaction ID of the reveal transaction,
and `N` is the index of the inscription in the reveal transaction.

The commit transaction commits to a tapscript containing the content of the
inscription, and the reveal transaction spends from that tapscript, revealing
the content on chain and inscribing it on the first sat of the input that
contains the corresponding tapscript.

Wait for the reveal transaction to be mined. You can check the status of the
commit and reveal transactions using  [the litecoinspace.org block
explorer](https://litecoinspace.org/).

Once the reveal transaction has been mined, the inscription ID should be
printed when you run:

```
ord wallet inscriptions
```

Parent-Child Inscriptions
-------------------------

Parent-child inscriptions enable what is colloquially known as collections, see
[provenance](../inscriptions/provenance.md) for more information.

To make an inscription a child of another, the parent inscription has to be
inscribed and present in the wallet. To choose a parent run `ord wallet inscriptions`
and copy the inscription id (`<PARENT_INSCRIPTION_ID>`).

Now inscribe the child inscription and specify the parent like so:

```
ord wallet inscribe --fee-rate FEE_RATE --parent <PARENT_INSCRIPTION_ID> --file CHILD_FILE
```

This relationship cannot be added retroactively, the parent has to be
present at inception of the child.

Sending Inscriptions
--------------------

Ask the recipient to generate a new address by running:

```
ord wallet receive
```

Send the inscription by running:

```
ord wallet send --fee-rate <FEE_RATE> <ADDRESS> <INSCRIPTION_ID>
```

See the pending transaction with:

```
ord wallet transactions
```

Once the send transaction confirms, the recipient can confirm receipt by
running:

```
ord wallet inscriptions
```

Sending Runes
-------------

Ask the recipient to generate a new address by running:

```
ord wallet receive
```

Send the runes by running:

```
ord wallet send --fee-rate <FEE_RATE> <ADDRESS> <RUNES_AMOUNT>
```

Where `RUNES_AMOUNT` is the number of runes to send, a `:` character, and the
name of the rune. For example if you want to send 1000 of the EXAMPLE rune, you
would use `1000:EXAMPLE`.

```
ord wallet send --fee-rate 1 SOME_ADDRESS 1000:EXAMPLE
```

See the pending transaction with:

```
ord wallet transactions
```

Once the send transaction confirms, the recipient can confirm receipt with:

```
ord wallet balance
```

Receiving Inscriptions
----------------------

Generate a new receive address using:

```
ord wallet receive
```

The sender can transfer the inscription to your address using:

```
ord wallet send --fee-rate <FEE_RATE> ADDRESS INSCRIPTION_ID
```

See the pending transaction with:
```
ord wallet transactions
```

Once the send transaction confirms, you can confirm receipt by running:

```
ord wallet inscriptions
```
