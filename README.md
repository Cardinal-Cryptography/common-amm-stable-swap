# ![Common logo](common_logo.svg "Common logo")

This repository contains an implementation of StableSwap contract for Common DEX.

## Versions

[ink! 4.3.0](https://github.com/paritytech/ink/tree/v4.3.0)
`cargo-contract` in version `3.2.0`

## License

Apache 2.0

### 🏗️ How to use - Contracts

##### 💫 Build

Use these [instructions](https://use.ink/getting-started/setup) to set up your ink!/Rust environment.
To build all contracts, run this command from the project root directory:

```sh
make build-all
```

##### 💫 Build verifiably

Given a deployed set of contracts with some code hashes, it's possible to check that the contract has been produced from a certain version of the source code in this repo (say a given commit). To do so:

1. `git checkout $COMMIT`
2. `make build-dockerized`.

You can also run `make build-dockerized` and ensure that the generated code hashes are the same as the ones found on chian.

The contracts will be deployed using the same docker image as the one used for this procedure, which smooths out indeterminism in ink! contract compilation.

The reason to build contracts with this command is to allow for _reproducible builds_ (ink! contracts' builds are not deterministic).

##### How to verify

Check out the repository at commit `TODO` (after deployment) and in the root of the project run the command above. This will output contracts' builds to `/target/ink` directory.

For every contract there's a separate folder in which you will find `<contract>.json` containing contract's metadata. One of the keys is `source.hash`. Compare that to the code hash of the on-chain contract.

##### 💫 Wrap

Use these [instructions](https://github.com/Cardinal-Cryptography/ink-wrapper#installation) to set up your `ink-wrapper` environment.
Once you have built your contracts, you can wrap them by running this command from the project root directory:

```sh
make wrap-all
```

You can also build and wrap the contracts in one step using:

```sh
make build-and-wrap-all
```

##### 💫 Run checks

Rust code checks and unit tests can be run from the root directory of the project:

```sh
make check-all
```

##### 💫 Run unit test

To manually run unit tests, use:

```sh
cargo test
```

##### 💫 Run DRink! tests

To run the DRink! test suite, execute the following command from the root directory of the project.

```sh
make all-drink
```

This will:

- Build and wrap your contracts.
- Run e2e tests, using DRink! environment.

##### 💫 Help

You can see a list of available `make` recipes by running:

```sh
make help
```

## Acknowledgement

The contracts here implement a constant-sum AMM based on the Curve and Ref Finance. Code has been adapted to work with ink! and be more easily integrated with already-existing Common AMM.
