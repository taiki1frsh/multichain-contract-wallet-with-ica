# Simple CosmWasm contract wallet with the feature of ICA and 1 of n multisig

*These codes is hugely from the repository of the Ethan Frey's [cw-ibc-demo](https://github.com/confio/cw-ibc-demo).*

**One contract address on the controller chain behave like an ICA with the support of the normal msg and 1 of n multisig-like feature.**
### Motivation

- To avoid huge amount of money by managing kinda 1 of n multisig enabled contract
- And simpler management of 1 of n multisig-like feature
- To simply implement the ICA features with more useful upgradability (arguably)
- To easily extend the feature of Account Abstraction
- Possible solution for callback function in the contract with IBC by using ack process for it

More features which can be added:

- Updating admin addresses requires more than half of the current admins keys’s signature
- Batch tx execution for saving gas
- More user-friendly change of the main account by contract migration to the other controller contract for the escape, or whatever you want

#### Callback logic

What I wanted to realise is actually very simple.  
In the ack packet msg, insert the `classback: bool` for the assertion whether to get callback.
```rust
pub enum PacketMsg {
    ..,
    ..,
    Balances { callback: bool }, // bool whether to get callback
}
```

This value is defined here:
```rust
pub enum ExecuteMsg {
    ..,
    CheckRemoteBalance {
        channel_id: String,
        callback: bool,
    },
    ..,
}
```

`CheckRemoteBalance` message does query the account information of the defined channel's address and update the `AccountData` by recering those data via ack response.   
What if I insert the sufficient data to execute the msg in the controller contract when the ack response returns from the host contract?
like this:
```rust
pub struct BalancesResponse {
    pub account: String,
    pub balances: Vec<Coin>,
    pub execute_callback: bool, // boolean value after the additional condition for callback
    // pub msg: ExecuteMsg::{RandomMsg}, <- the message object to be triggered as a call back fn 
}
```

For now, I didn't implement `msg` like data in the response as the message for a callback fn, but if the current implementation works as intended, there would be no problem with it, I expect.

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
