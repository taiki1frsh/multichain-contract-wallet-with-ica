# Simple CosmWasm contract wallet with the feature of ICA and 1 of n multisig

These codes is hugely from the repository of the Ethan Frey's [cw-ibc-demo](https://github.com/confio/cw-ibc-demo).

### Motivation

- To avoid huge amount of money by managing kinda 1 of n multisig enabled contract
- To simply implement the ICA features with more useful upgradability
- To easily extend the feature of Account Abstraction

_Example IBC enabled contracts along with full stack integration tests_

This package demos how to write a simple pair of IBC-enabled contracts
that speak to each other. It includes unit tests on each contract
in Rust, as well as full stack integration tests on two live blockchains
using [CosmJS](https://github.com/cosmos/cosmjs) and the
[TS-Relayer](https://github.com/confio/ts-relayer).

## Design

This is a simple set of Interchain Account (ICA)-like contracts.
`simple-ica-host` will receive messages from a remote connection
and execute them on it's chain. `simple-ica-controller` will
send messages from the original chain and get the results.

The main difference between this and ICA is the use of one
unordered channel rather than multiple ordered channels. We
also use a different payload with a CosmWasm/JSON focus.

This could be the basis of writing full ICA compatible contracts,
but the main focus here is the ability to showcase how to write
and test IBC contracts in general.

## Rust Contracts

The package `simple-ica` holds common types and functionality
used in both contracts. The concrete logic is stored
in `simple-ica-host` and `simple-ica-controller`.

To ensure they are proper, run the following in the repo root:

```shell
cargo build
cargo fmt
cargo clippy --tests
```

## Unit Tests

All unit tests are in Rust and assume a mocked out environment.
They don't actually send packets between contracts in any way,
but return a fully mocked response. This can run through many
code paths and get a reasonable level of confidence in the basic
logic. However, you will need to run through full-stack
integration tests to actually have any confidence it will work
as expected in production.

To ensure they are proper, run the following in the repo root:

```shell
cargo test
```

## Integration Tests

TODO
