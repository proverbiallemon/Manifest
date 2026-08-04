# Manifest

A load order analyzer and sorter for [Ship of Harkinian](https://www.shipofharkinian.com/) mods, in the spirit of LOOT.

Manifest reads every mod archive in your library, builds an asset-level map of which mods overwrite which, proposes a corrected load order with a plain-English reason for every move, and warns you about the problems no ordering can fix.

## Why

Ship of Harkinian resolves mod conflicts by load order alone: when two mods ship the same asset, the one loaded later wins, silently. With a big library it is nearly impossible to know why a texture is not showing up or which of your three Mirror Shield variants is actually in the game. Manifest makes the invisible visible. On a real 566-mod library it identified 1,282 contested assets on its first run.

## What it does

- Lists the contents of both `.otr` (MPQ) and `.o2r` (ZIP) mod archives
- Builds a conflict map: every contested asset, every mod that provides it, and the current winner
- Proposes a fixed load order using a specificity rule: the more focused mod wins over the broad pack
- Explains every proposed move ("moved after X: its 6 assets overlap X's 214 and the more specific mod wins")
- Warns about mods that do nothing (fully overridden), mod groups where only one can ever show, unreadable archives, and duplicate installs
- Writes the corrected order back to `shipofharkinian.json`, touching only the mod list and preserving every other setting, atomically

## Install the app

Grab the latest GUI release from the [releases page](https://github.com/proverbiallemon/Manifest/releases):

- **macOS**: `brew install --cask pocketbeardev/manifest/manifest`, or the `.dmg` (universal, signed and notarized); open it and drag Manifest to Applications
- **Windows**: the `-setup.exe` installer, or the `.msi`. Neither is code signed yet, so SmartScreen will warn; choose "More info" then "Run anyway"
- **Linux**: the `.AppImage` (`chmod +x`, then run it), or the `.deb`/`.rpm` for your package manager

The `manifest` CLI below is built from source for now; the app and the CLI read and write the same config and pins files.

## Usage

```
manifest scan            # summary of conflicts and warnings
manifest scan --json     # full machine-readable report
manifest sort --dry-run  # preview the proposed order with reasons
manifest sort --write    # apply it
manifest explain <mod>   # why does this mod sort where it does?
```

Exit codes: 0 clean, 3 conflicts or warnings found, 1 error.

Paths are auto-detected per platform; override with `--config` and `--mods-dir`.

## Pins

Pins let you override the sorter for specific mods. Create
`manifest-pins.json` next to `shipofharkinian.json`:

```json
{
  "schema_version": 1,
  "top": ["SomeOverhaul"],
  "bottom": ["MyFavoriteSkin"]
}
```

Mods in `top` load first; mods in `bottom` load last, and later always wins.
Pinned mods are never reordered by the heuristics. Pin names that do not
match an installed mod are ignored. Every command picks the file up
automatically; there is nothing to enable.

## Safety

Mod archives are treated as untrusted input throughout: every read is bounds-checked and malformed files produce typed errors, never crashes. Config writes are atomic, modify only the `EnabledMods` entry, and are refused entirely if the existing config cannot be parsed. Manifest never modifies mod files themselves.

## Status

The core library, CLI, and desktop app are done. Next up is integration into [Sailswift](https://github.com/proverbiallemon/Sailswift), the macOS mod manager for Ship of Harkinian.

## Building

```
cargo build --release
cargo test
```

## License

GPL-3.0. See [LICENSE](LICENSE).
