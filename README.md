# Solarium

Solarium is a framework for building Solana programs with a more idiomatic Rust experience.

## Usage

Install the Solarium CLI:

```bash
cargo install solarium-cli
```

Run the tests:

```bash
solarium test
```

Run the tests and detach the validator after the tests are finished:

```bash
solarium test --detach
```

Build the Solarium workspace:

```bash
solarium build
```

Build the Solarium workspace and start the local validator:

```bash
solarium dev
```

Deploy the Solarium workspace:

```bash
solarium deploy
```

## Program backends

The default backend uses solana-program. A real Pinocchio backend is available behind the
"pinocchio" feature; it uses Pinocchio account views, entrypoint parsing, CPIs, rent, and resizing.
The two backend features are mutually exclusive.

A program crate can forward the selection like this:

~~~toml
[dependencies]
solarium-program = { path = "../../crates/solarium-program", default-features = false }

[features]
default = ["solana-program-backend"]
solana-program-backend = ["solarium-program/solana-program-backend"]
pinocchio = ["solarium-program/pinocchio"]
~~~

Adjust the path for your project. If inheriting the dependency from a workspace, disable its
defaults in the workspace dependency definition as well.

~~~bash
# Existing backend
cargo check -p solarium-example

# Pinocchio
cargo check -p solarium-example --no-default-features --features pinocchio
cargo build-sbf --arch v3 --manifest-path examples/solarium-example/Cargo.toml -- --no-default-features --features pinocchio
~~~

The high-level Account, Signer, Program, data guards, initialization methods, and program macros
remain available. Instruction discriminators, Borsh layouts, PDA seeds, IDL, and generated clients
are unchanged. Existing legacy-only programs do not have to opt into new features.

Code shared between backends should access account metadata with methods such as info.key(),
info.owner(), info.is_signer(), and info.set_lamports(...), importing the Solarium prelude.
Writing bytes or resizing through Pinocchio requires a mutable Account. Raw AccountInfo fields
and Solana-specific CPI types remain legacy-only; Pinocchio-specific adapters can use
info.as_view(). Use solarium_program::msg for logging. Its Pinocchio implementation uses typed
logging; pass String values as .as_str().

Backend switching is a compile-time build choice, not a deployment operation. Test new artifacts
on a disposable local validator with a throwaway wallet before any public-cluster deployment.

## Documentation (mdBook)

The tutorial lives under `docs/` and is built with mdBook.

Build locally:

```bash
cargo install mdbook
mdbook build docs
```

Serve locally with live reload:

```bash
mdbook serve docs --open
```
