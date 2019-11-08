# WIP Ledger HW wallet Bitcoin API

Current functionality:
* fetch xpubs from ledger (see the [fetch_addresses](https://github.com/sgeisler/ledger-bitcoin/blob/master/examples/fetch_addresses.rs) example)

Not yet implemented functionality:
* show address on device
* sign single sig TX
* sign multisig TX
* choosing a device if multiple ones are connected (needs upstream patch)

The documentation of the protocol can be found [here](https://ledgerhq.github.io/btchip-doc/bitcoin-technical.html).