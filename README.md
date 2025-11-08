# arithmetic-coding

An implementation of the [arithmetic coding][1] compression algorithm based on
[mathematicalmonk's information theory playlist][2].

[1]: https://en.wikipedia.org/wiki/Arithmetic_coding
[2]: https://www.youtube.com/playlist?list=PLE125425EC837021F

## Tests

### Logging

To run a test case with verbose logging:

```bash
RUST_LOG=debug cargo test <test-case> -- --nocapture
```

### Coverage

You can generate a code coverage report with the [tarpaulin][3] tool:

[3]: https://crates.io/crates/cargo-tarpaulin

```bash
cargo tarpaulin --out Html --output-dir /tmp && xdg-open /tmp/tarpaulin-report.html
```
