# Samples

Sample `.strut` files used by the docs, Studio shell, and validation examples.

## Validate

```powershell
cargo run -p strut-format --example validate -- samples/login-button.strut
cargo run -p strut-format --example validate -- samples/minimal-bot.strut
cargo run -p strut-format --example validate -- samples/game-mascot.strut
```

## Regenerate

```powershell
cargo run -p strut-format --example write_sample -- samples/login-button.strut
cargo run -p strut-format --example write_sample -- samples/minimal-bot.strut
cargo run -p strut-format --example write_sample -- samples/game-mascot.strut
```
