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
* **Top (`t`):** Global metadata properties defined at the top of a document.
* **Graphic (`g`):** Embedded visual media (Images / Figures).

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

---

### 2. Inline Elements (The Micro State)
Inline Elements exist within a line of text. Since they do not end at a line break, they must be collapsed manually using the **Annihilator (`..`)** particle.

To maintain perfect legibility, the command particle "clings" to the left word but separates itself from the formatted content with a space.

* **Style & Formatting (Charm):** `This is a .c charming.. experience.`
* **Inline Code (Strange):** `Run the .s cargo build.. command.`
* **Superscript & Subscript (Up/Down):** * `a.u 2.. + b.u 2.. = c.u 2..`
  * `H.d 2..O is essential for life.`

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

## 🪐 Join the Subatomic Universe

Quarkup is currently in its pre-alpha orbital state. Contributions to the parser or link-refinement mechanics are highly welcome!

## License

[MIT](LICENSE)
