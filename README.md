# vial-keymap-pdf-export

Connects to a VIA/Vial keyboard over USB and prints every layer of its
current keymap to a PDF — one page (or several, stacked) per layer, laid out
in the keyboard's real physical shape, with keycap labels translated into
your keyboard layout (German, French, Italian, Spanish, ... — see
[Adding a language](#adding-a-language)).

## Credits

Built on two projects by [Stephan Rumswinkel (srwi)](https://github.com/srwi):

- **[qmk-via-api](https://github.com/srwi/qmk-via-api)** — used as a normal
  Cargo dependency for the VIA HID protocol (device scanning, raw HID
  read/write, keycode constants).
- **[keypeek](https://github.com/srwi/keypeek)** — several modules were
  **copied and adapted** from here (not just referenced): the Vial protocol
  handshake, the KLE/physical-layout parser, and the large QMK
  keycode-to-label tables. keypeek itself is untouched by this project; the
  code lives here as its own copy.

Because of that direct code reuse, this project is licensed **GPL-3.0**,
same as keypeek and qmk-via-api — see [LICENSE](LICENSE).

## What you need

- **Rust** (`cargo`) to build it — [rustup.rs](https://rustup.rs)
- A **C toolchain**, needed by the `hidapi` dependency:
  - Linux: usually already present (`gcc`/`clang`)
  - Windows: comes with the MSVC Build Tools, which `rustup`'s default
    Windows install pulls in automatically
- **Linux only:** read/write permission on `/dev/hidraw*` for your VIA/Vial
  keyboard. If Vial's own desktop app or QMK Toolbox already works for you,
  you have this. Otherwise you need a udev rule granting your user access
  (search "qmk udev rules" — most keyboard projects ship one).
- Nothing else — the font is compiled into the binary, no system font or
  extra install required, on either OS.

## Build

```bash
git clone https://github.com/kolbenhans/vial-keymap-pdf-export
cd vial-keymap-pdf-export
cargo build --release
```

Produces two binaries — `vial-keymap-pdf-export` (CLI) and `gui` (GUI) —
under `target/release/` (Linux) or `target\release\` (Windows, `.exe`).

## GUI

```bash
./target/release/gui          # Linux
.\target\release\gui.exe      # Windows
```

Pick a keyboard, language, portrait/landscape, layers per page, and where
to save — then hit **Export PDF**. Same options as the CLI flags below, no
terminal required.

## CLI

Scriptable, and useful for batch-exporting every connected keyboard at
once (the GUI exports one device at a time).

```bash
# Linux
./target/release/vial-keymap-pdf-export --lang=de_DE

# Windows (PowerShell / cmd)
.\target\release\vial-keymap-pdf-export.exe --lang=de_DE
```

With no arguments it lists every connected VIA/Vial keyboard and writes a
PDF for each (`<Product Name>.pdf`, in whichever directory you ran it from).
Pass a number to target just one:

```bash
./target/release/vial-keymap-pdf-export --lang=de_DE 0
```

### Options

| Flag | Default | What it does |
|---|---|---|
| `--lang=<code>` | `en_US` | Keyboard layout to translate keycap labels into. Run with an unknown code to see the available ones. |
| `--portrait` | off | Rotates the page 90° for portrait printing instead of the natural (wide) landscape shape. |
| `--layers-per-page=<N>` | `1` | Stacks N layers onto each page instead of one page per layer. |
| `<device index>` | all devices | Only export the Nth device from the device list. |

Example: `--lang=de_DE --portrait --layers-per-page=2 0`

## Adding a language

Every language is a single JSON file in [`languages/`](languages/). All of
them are compiled into the binary, so it works standalone with zero setup
— but you don't need to rebuild to add one: drop a `languages/xx_YY.json`
file into the same folder as the executable (that's what the release
tarball/zip/AppImage already ship next to the binary) and it's picked up
immediately, no rebuild.

1. Name the file `xx_YY.json` (locale code — e.g. `nl_NL.json`,
   `pl_PL.json`). The filename minus `.json` becomes the `--lang=` code.
2. Copy an existing file as a starting point — `languages/de_DE.json` for a
   full example, `languages/en_US.json` for the minimal/empty template.
3. Fill in only what actually differs from raw US labels.
4. Drop it next to the built executable (`target/release/languages/` if
   you're building from source — symlink `languages/` there once for a
   live dev loop: `ln -s ../../languages target/release/languages`).

```json
{
  "name": "Polish (Programmers)",
  "base":        {},
  "shifted":     {},
  "altgr":       { "KC_E": "ę" },
  "shift_altgr": { "KC_E": "Ę" }
}
```

- `base` — what a key sends unshifted, when its *physical position* differs
  from US (e.g. the Y/Z swap on German QWERTZ).
- `shifted` — what `Shift + <keycode>` produces, when your keymap has a key
  explicitly programmed as e.g. `S(KC_8)`.
- `altgr` — same, for `AltGr + <keycode>` combos.
- `shift_altgr` — same, for `Shift + AltGr + <keycode>` combos (its own
  combo, not covered by `shifted`/`altgr` alone — e.g. Polish Programmers'
  Ę on Shift+AltGr+E).

Keys are QMK keycode names (`KC_*`), e.g. `KC_A`, `KC_MINUS`,
`KC_LEFT_BRACKET` — see [QMK's keycode reference](https://docs.qmk.fm/keycodes)
for the full list. Leave a table empty (or the whole file minimal, like
`languages/en_US.json`) for a layout with no deviations from the raw US
labels.
