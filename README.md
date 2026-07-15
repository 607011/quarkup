# Quarkup — The Subatomic Markup Language

Quarkup is an extremely lightweight, predictable, and context-aware markup language designed for speed and human-readability. Inspired by the principles of **Quantum Chromodynamics (QCD)**, Quarkup replaces complex bracket hierarchies and bulky tags with an elegant system of **Quarks** (fundamental formatting particles), **Leptons** (light structural elements), and **Annihilators** (field stoppers).

Unlike other markup languages that suffer from visual noise, Quarkup looks like natural, unformatted text while remaining incredibly simple and lightning-fast to parse.

---

## 🌌 The Core Philosophy: Subatomic Minimalism

In Quarkup, we do not have "tags" or "elements". We only have **Particles** and **Forces**.

### 1. The Quark Flavors (Structural & Styling Operators)
Formatting behaviors are controlled by a minimal set of flavor letters:
* **Up (`u`):** Pulls text upwards (Headings / Superscript).
* **Down (`d`):** Pushes text downwards (Subscript / Footnotes).
* **Charm (`c`):** Enhances aesthetic appeal (Italics / Emphasis).
* **Strange (`s`):** Represents foreign, unformatted matter (Code / Listings).
* **Muon (`m`):** A heavier cousin of the electron — adds visual mass (Bold / Strong).
* **Top (`t`):** Global metadata properties defined at the top of a document.
* **Graphic (`g`):** Embedded visual media (Images / Figures).
* **Lattice (`l`):** A crystalline grid structure — as a block it builds Tables, as an inline particle it forms Links.

### 2. The Lepton Family (Lists & Structural Flows)
Lists are powered by light, fundamental leptons:
* **Neutrino (`n`):** An uncharged particle used for unordered bullet lists.
* **Electron (`e`):** A countable, charged particle used for ordered, numbered lists.

### 3. The Universal Command Trigger
Every Quarkup command follows one unyielding, predictable rule:
> **`[Dot]` `[Particle-Letter(s)]` `[Space]`**

If a dot is followed by a letter and a space (e.g., `.u ` or `.n `), it is an active operator. If it is not followed by a space (e.g., `file.txt` or `e.g.`), it remains a dormant, natural punctuation mark. No backslash escaping required!

---

## 🛠️ Syntax Guide

Quarkup classifies elements into two states of matter: **Block Elements** and **Inline Elements**.

### 1. Block Elements (The Macro State)
A Block Element starts at the beginning of a line and automatically collapses (ends) at the next line break (`\n`).

#### Headings (Quark Stacking)
Rather than introducing arbitrary heading markers, Quarkup increases the "mass" of the `Up` quark by stacking the letters. More quarks yield a smaller, denser heading level.
* `.u Heading 1`  →  `<h1>Heading 1</h1>`
* `.uu Heading 2` →  `<h2>Heading 2</h2>`
* `.uuu Heading 3` → `<h3>Heading 3</h3>`

#### Lists & Deep Nesting (Lepton Stacking)
Sequential lepton blocks of the same type are automatically grouped together. Stacking the particle letters sinks them deeper into the hierarchical structure, creating sub-lists.
* **Unordered (Neutrinos):**
  ```text
  .n Main Point
  .nn Sub-point level 2
  .nnn Sub-point level 3
  .n Back to Main Point
  ```
* **Ordered (Electrons):**
  ```text
  .e First step
  .e Second step
  ```

#### Metadata & Images
* `.t title My First Quarkup Document`
* `.g terrace.jpg A beautiful view from the terrace`

#### Tables (Lattices)
A `.l` block opens a table; it closes on a solitary Annihilator (`..`) on its own line. Each line inside is one row, with cells separated by `;`.

Rows can carry an optional type prefix:
* `h:` — Header row (wrapped in `<thead>`)
* `f:` — Footer row (wrapped in `<tfoot>`)
* `s:` — Section row (a single centered cell spanning the full table width, e.g. as a group divider)
* *(no prefix)* — Regular body row

Within Header, Body, and Footer rows, a cell containing only `>` merges into the cell to its left (colspan), and a cell containing only `_` merges into the cell above it (rowspan).

Cell content can carry an alignment marker as a prefix — left is the default and needs no marker:
* `.> ` — right-aligned
* `.^ ` — centered

```text
.l
h: Product ; Qty  ; Price
Widget     ; .^ 3 ; .> 9.99
Gadget     ; .^ 1 ; .> 199.00
f: Total   ; >     ; .> 208.99
..
```

renders a table with a header row, two body rows with centered quantities and right-aligned prices, and a footer row whose first two columns merge into a single "Total" cell.

#### Conditional Rendering (Bottom)
A `.b condition` block includes its enclosed blocks only when `condition` matches the active defines, then closes on a solitary Annihilator (`..`). Defines are supplied via the CLI's repeatable `-d KEY=VALUE` flag (bare `-d KEY` counts as `KEY=true`), or via the "Defines" field in the [web playground](web/index.html).

```text
.b target=web
.u Web-only heading
This paragraph only appears in the browser build.
..
```

Compiled with `-d target=web`, the heading and paragraph are included; compiled without it, the whole block — and its condition line — disappear without a trace. Prefix the condition with `!` to negate it, e.g. `.b !target=web` renders only when `target` is *not* set to `web`.

---

### 2. Inline Elements (The Micro State)
Inline Elements exist within a line of text. Since they do not end at a line break, they must be collapsed manually using the **Annihilator (`..`)** particle.

To maintain perfect legibility, the command particle "clings" to the left word but separates itself from the formatted content with a space.

* **Style & Formatting (Charm):** `This is a .c charming.. experience.`
* **Bold (Muon):** `This is .m important.. news.` — nests freely with Charm in either direction, e.g. `.m bold with .c nested italic.. inside..`
* **Inline Code (Strange):** `Run the .s cargo build.. command.`
* **Superscript & Subscript (Up/Down):** * `a.u 2.. + b.u 2.. = c.u 2..`
  * `H.d 2..O is essential for life.`
* **Conditional (Bottom):** `This feature is available .b target=web|only in your browser...` keeps the phrase when compiled with `-d target=web` and drops it (surrounding text and punctuation stay put) otherwise. Negate with `!`, same as the block form. Note the `condition|text` form only triggers when `.b` is *not* the first token on the line — a line-opening `.b` is always parsed as the block form described above.

---

### 3. Code Listings (Pre-formatted Blocks)
For multi-line source code, we use the `.s` (Strange) block. It opens with `.s [language]` and captures all text exactly as written — ignoring all inner punctuation and formatting — until it encounters a solitary Annihilator (`..`) on its own line.

```text
.s rust
fn main() {
    let ans = 42;
    println!("The answer is: {}", ans);
}
..
```

---

## ⚡ The Rust Implementation

This repository features a robust, **Zero-Copy Parser** written in pure Rust. It features:
1.  **Zero-Heap Allocations:** The Lexer yields string slices (`&str`) pointing directly into your original document, making it blisteringly fast and light on memory.
2.  **Context-Aware Parser:** A hand-written predictive parser that effortlessly distinguishes between block-level commands, inline styling, nested lists, and raw code listings.
3.  **Clean HTML5 Output:** Transforms your Quarkup document into semantic, standard-compliant HTML5 without auxiliary wrapping tags.

### Running the Demo via CLI Pipeline

Since the compiler reads directly from standard input (`stdin`), you can effortlessly pipe your `.qu` files directly into it:

```bash
cargo run < example/demo.qu > demo.html
```

You can embed referenced images using Data URLs using the `--monolithic` switch:

```bash
cargo run -- --monolithic < example/demo.qu > demo.html
```

### Example Input to Output

**Input (`test.qu`):**
```text
.u Welcome to Quarkup
This is a molecule of H.d 2..O.

Here is what we need to do:
.e Clone repo
.e Run tests

.s rust
fn hello() {
    println!("Hello, World!");
}
..
```

**Generated HTML:**
```html
<h1>Welcome to Quarkup</h1>
<p>This is a molecule of H<sub>2</sub>O.</p>
<p>Here is what we need to do:</p>
<ol>
<li>Clone repo</li>
<li>Run tests</li>
</ol>
<pre class="language-rust">fn hello() {
    println!("Hello, World!");
}</pre>
```

#### Result in browser

This is how the [sample document](example/demo.qu) looks like in a web browser:

<img width="3620" height="5878" alt="grafik" src="https://github.com/user-attachments/assets/7f9d5ffb-d04c-40e9-a203-165154a12a4b" />


---

## 🌐 Standalone Web App (WASM)

The compiler also runs entirely client-side in the browser via WebAssembly — no server, no data leaves the machine. The frontend lives in [`web/index.html`](web/index.html) and talks to a `quarkup` crate built with `crate-type = ["cdylib", "rlib"]` (see [`Cargo.toml`](Cargo.toml) and [`src/wasm.rs`](src/wasm.rs)).

Build the wasm module and JS bindings:

```bash
./web/build.sh
```

This requires the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`) and a matching `wasm-bindgen-cli` version (`cargo install wasm-bindgen-cli --version <version-of-wasm-bindgen-in-Cargo.toml>`).

Note that LaTeX rendering (`.s math` / inline `.s math ...`) works differently in the browser: `mathjax-svg-rs` renders by spawning an OS thread, which `wasm32-unknown-unknown` cannot support, so the wasm target compiles without that dependency and instead emits a placeholder holding the raw LaTeX source. `web/index.html` typesets those placeholders client-side with the vendored [KaTeX](web/vendor/katex/) (MIT-licensed, bundled locally under `web/vendor/katex/`) once the preview loads. The CLI is unaffected and keeps using `mathjax-svg-rs` directly.

Then serve the `web/` directory with any static HTTP server (ES modules require `http://`, not `file://`):

```bash
python3 -m http.server -d web 8080
```

and open `http://localhost:8080`.

---

## 🪐 Join the Subatomic Universe

Quarkup is currently in its pre-alpha orbital state. Contributions to the parser or link-refinement mechanics are highly welcome!

## License

[MIT](LICENSE)
