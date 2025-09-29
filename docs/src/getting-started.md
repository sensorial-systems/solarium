# Getting Started

## Prerequisites

- Rust toolchain (stable)
- Solana toolchain if you plan to run a local validator
- `mdbook` if you want to build these docs locally

Install mdBook:

```bash
cargo install mdbook
```

## Install Solarium CLI

```bash
cargo install solarium-cli
```

## Build and Test a Workspace

From the repository root:

```bash
# Build the Solarium workspace
solarium build

# Run tests
solarium test

# Run tests and keep the validator running afterwards
solarium test --detach

# Start a local validator and rebuild on changes
solarium dev

# Deploy the workspace (configure your Solana CLI as needed)
solarium deploy
```
