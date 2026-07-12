# soksak-kit-terminal-conformance

Build-time facade for the conformance interface owned by `soksak-contract-terminal`.

## What it provides

- Re-exports of the contract's `MirrorUnderTest`, canonical `ScreenState`, fixtures, and assertions.
- Named compatibility functions for the seven declared contract cases.
- `conformance_suite!`, which generates the declared-state tests for an engine mirror.
- An Alacritty-backed `Screen` for provider diagnostics. It is not a correctness authority.

## Correctness authority

The contract repository owns the corpus, normalization rules, declared reference states, and
expected outcomes. A mirror interprets the contract stream into canonical `ScreenState`; its warm
and cold paint are fed into a fresh mirror and compared with the same declared state. No engine
implementation supplies universal expected state.

The compatibility functions retain the established test names while delegating each verdict to
`soksak-contract-terminal::assert_conforms`.

## How a unit uses it

Add the kit as a dev-dependency and expose the complete `MirrorUnderTest` surface: `new`, `feed`,
`resize`, `rehydrate`, `cold_paint`, `suppressed_replies`, `screen_state`, and `cursor_style`. The
test crate can then generate the suite:

```rust
soksak_kit_terminal_conformance::conformance_suite!(EngineMirror);
```

The macro runs the seven declared fixtures plus the contract-owned cursor and resize cases. A
missing canonical-state or cursor-state method fails at compile time.

## Diagnostic renderer

`judge::Screen` renders bytes with `alacritty_terminal` and captures terminal replies. It remains a
provider diagnostic surface and keeps the existing `CellSnap`, `ModeSnap`, and `ColorSnap` formats.
The conformance functions do not read expected state from it.

## Test

```sh
cargo test
```

## License

The diagnostic renderer uses `alacritty_terminal` under Apache-2.0. See `LICENSE` and
`THIRD-PARTY-NOTICES` for the complete notice. The kit is a dev-dependency and does not put that
engine into a unit's distributable.
