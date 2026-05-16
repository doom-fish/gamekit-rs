# gamekit-rs

Safe Rust bindings for Apple's `GameKit` framework — Game Center, leaderboards, achievements, and multiplayer on macOS.

## Requirements

- macOS 12.0+
- Xcode / Swift toolchain

## Usage

```rust
use gamekit::LocalPlayer;

let player = LocalPlayer::local()?;
println!("authenticated: {}", player.is_authenticated);
```

## License

MIT OR Apache-2.0
