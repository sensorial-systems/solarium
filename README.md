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
