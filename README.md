# rdiff implementation in Rust #

Rust equivalent of rdiff C implementation. File diffing algorithm uses rsync
rolling checksum (as a weak hash) and Blake2b (as a strong hash). Weak hash
needs to be fast and inexpensive (higher random collision probability but low
enough to be effective at quickly filtering out false-positives without too
many collisions) and strong hash - a more expensive one but with a very low
random collision probability.

`rdiff-rust` could be used to efficiently send file updates to a remote location
by saving throughput. It is due to the fact that instead of sending whole
files, signature and delta files are exchanged between client and server.

Currently signature and delta part is implemented. Patching - not yet.

## Building ##

```bash
# Clone repo
git clone https://github.com/TomaszAugustyn/rdiff-rust.git
cd rdiff-rust

# Build with cargo
cargo build

cd target/debug/

# Generate signature file from unchanged (old) file
./rdiff-rust signature /tmp/old /tmp/signature

# Generate delta file from signature file and changed (new) file
./rdiff-rust delta /tmp/signature /tmp/new /tmp/delta
```

## Tests ##

```bash
cargo test
```
