# Solarium

Solarium is a framework for building Solana programs with a more idiomatic Rust experience.

## Usage

Install the Solarium CLI:

```bash
cargo install solarium-cli
```

Programs build with the platform tools of `cargo build-sbf`. The client side (solarium-client,
and so a program's tests) builds on solana-client 4.3, which needs rustc 1.97.1 or newer.

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

Deploy the Solarium workspace to the cluster the Solana CLI is configured for:

```bash
solarium deploy
```

Deploy to a named cluster instead, optionally one program of the workspace:

```bash
solarium deploy --url https://api.devnet.solana.com
```

`dev` and `test` take no `--url`: they deploy to the local validator they start.

## Program backends

The default backend uses solana-program. A real Pinocchio backend is available behind the
"pinocchio" feature; it uses Pinocchio account views, entrypoint parsing, CPIs, rent, and resizing.
The two backend features are mutually exclusive.

Since solana-program 5, both backends are built on the same no_std crates: Pubkey is
solana-address's Address, which is Pinocchio's too, and ProgramError is solana-program-error's
on both, so keys and errors pass between them without conversion.

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

## Fixed addresses and instruction numbers

A program already deployed, or one that other programs call back into, can keep the wire format it
has instead of taking Solarium's defaults:

~~~rust
#[program(id = "SLoTSdnmBH5KtNJjhEYw1MeWTKAfRnFfQTTcpgwRn2Q")]
impl Slots {
    // Answers to the little-endian u64 16 rather than sha256("global:request_bet").
    #[instruction(discriminator = 16)]
    pub fn request_bet(&self, /* ... */) -> Result<()> { /* ... */ }

    // Also reached by the tag the delegation program calls back with. Clients send 3.
    #[instruction(discriminator = 3, alias = "global:process_undelegation")]
    pub fn undelegate(&self, /* ... */) -> Result<()> { /* ... */ }
}
~~~

- `id` pins the program id in the source. Without it the id is read from the workspace's
  `target/deploy/<crate>-keypair.json`, which is generated if missing.
- A `discriminator` or `alias` is an integer (the u64 the instruction starts with) or a string,
  hashed as the defaults are. Two methods answering to the same one is a compile error.
- Bytes after an instruction's arguments are left unread, so a program calling back with extra
  data of its own still reaches the method.

An instruction whose account list runs on takes the tail as its last account parameter,
`extra: &Remaining<'a>`, which derefs to a slice of whatever followed the named accounts.

Each instruction is entered through a function of its own, which is handed the account slice and
into which the method is inlined. On sBPF this keeps every instruction in its own 4 KiB stack
frame, and lets a function the method calls once be inlined too (`#[inline(always)]`), finding
its accounts in the slice rather than being passed each one — past three accounts, every one
passed costs instructions at the call.

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
