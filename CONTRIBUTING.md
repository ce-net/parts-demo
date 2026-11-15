# Contributing to parts-demo

Thanks for helping. The rules are short:

1. **Open an issue first** for anything bigger than a typo or an obvious bug fix, so we can agree on
   the shape before you write code. Bug reports need the exact command, what you expected and what
   happened (the full error text).
2. **Small commits.** One change per commit, with a message that says what changed and why. Keep pull
   requests focused; several small ones beat one large one.
3. **Tests must pass.** Run the test command below before you push; CI runs the same command and a
   pull request is merged only when CI is green. A bug fix comes with a test that fails without it.

```
cargo test --all-targets
```

By contributing you agree that your contribution is licensed under the Apache License, Version 2.0,
the same license as the project (see LICENSE).
