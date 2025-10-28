### Zero-copy accounts

Zero-copy accounts store a small fixed header (Borsh) followed by an opaque payload in raw account bytes. This avoids re-serializing large vectors and enables efficient, chunked writes and reads.

- Define a header struct (no Vec), keep payload length in the header.
- Grow account data in steps (≤10_240 bytes per instruction) to respect Solana limits.
- Write chunks by offset directly into the payload slice. Compute checksums over the payload.

The `file-upload-example` demonstrates:
- `initialize`: create header (payload_len = 0)
- `grow`: reallocates the account, increases `payload_len`
- `upload_chunk`: writes into payload offset
- `check_crc`: verifies checksum over payload

