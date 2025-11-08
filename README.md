# arithmetic-coding

An implementation of the [arithmetic coding][1] compression algorithm based on
[mathematicalmonk's information theory playlist][2].

[1]: https://en.wikipedia.org/wiki/Arithmetic_coding
[2]: https://www.youtube.com/playlist?list=PLE125425EC837021F

## Tests

To run a test case with verbose logging:

```bash
RUST_LOG=debug cargo test <test-case> -- --nocapture
```
