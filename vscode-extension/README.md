# Quarkup for VS Code

Syntax highlighting and a live preview for [Quarkup](../README.md), the subatomic markup language, right inside VS Code.

## Features

- **Syntax highlighting** for `.qu` files — all Quark flavors (headings, bold, italic, sub/superscript, code, links, tables, conditionals), per-level heading colors, comments, and embedded language highlighting inside `.s <language>` code blocks (Rust, JS/TS, Python, JSON, HTML, CSS, C/C++, Go, Ruby, YAML, shell, SQL, Markdown, XML, Java, C#, PHP, TOML — anything else still gets a plain raw-block style).
- **Live preview** — compiles entirely client-side via the same WebAssembly build used by the [standalone web app](../web/), no network access and no external compiler process. Updates as you type.
  - `Quarkup: Open Preview` (`Ctrl+Shift+V` / `Cmd+Shift+V`)
  - `Quarkup: Open Preview to the Side` (`Ctrl+K V` / `Cmd+K V`), also available as a button in the editor toolbar for `.qu` files

## Known limitations

- The preview always compiles with no active `-d` defines and the default HTML template — `.b` conditionals that depend on a define will render as if compiled with plain `cargo run` (no flags). Exposing defines/template overrides in the preview UI is a possible future addition.
- LaTeX rendering (`.s math`) is typeset client-side with the vendored [KaTeX](../web/vendor/katex/), same as the web app — see the [main README](../README.md#-standalone-web-app-wasm) for why the wasm build can't use `mathjax-svg-rs` directly.

## Development

Requires the same Rust/wasm toolchain as the [web app](../web/): the `wasm32-unknown-unknown` target and a `wasm-bindgen-cli` version matching the `wasm-bindgen` crate in [`../Cargo.toml`](../Cargo.toml).

```bash
npm install
npm run build-assets   # builds the wasm compiler + copies it and KaTeX into media/
```

Then open this directory in VS Code and press **F5** to launch an Extension Development Host with the extension loaded — open any `.qu` file there to try it (e.g. [`../example/demo.qu`](../example/demo.qu)).

To build an installable package:

```bash
npm run package         # produces quarkup-<version>.vsix
```

Install it via VS Code's "Extensions: Install from VSIX..." command.

### Regenerating the grammar test fixtures

[`scripts/test-grammar.js`](scripts/test-grammar.js) tokenizes [`scripts/sample.qu`](scripts/sample.qu) with `vscode-textmate`/`vscode-oniguruma` — the same libraries VS Code uses internally — and prints the resulting scopes. Useful for checking grammar changes without a running VS Code instance:

```bash
npm install --no-save vscode-textmate vscode-oniguruma
node scripts/test-grammar.js
```

## License

[MIT](LICENSE)
