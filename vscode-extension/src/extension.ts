import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';

const DEBOUNCE_MS = 200;

export function activate(context: vscode.ExtensionContext): void {
    context.subscriptions.push(
        vscode.commands.registerCommand('quarkup.showPreview', () => {
            openPreview(context, vscode.ViewColumn.Active);
        }),
        vscode.commands.registerCommand('quarkup.showPreviewToSide', () => {
            openPreview(context, vscode.ViewColumn.Beside);
        })
    );
}

export function deactivate(): void {
    for (const panel of panelsByDocument.values()) {
        panel.dispose();
    }
    panelsByDocument.clear();
}

const panelsByDocument = new Map<string, QuarkupPreviewPanel>();

function openPreview(context: vscode.ExtensionContext, column: vscode.ViewColumn): void {
    const editor = vscode.window.activeTextEditor;
    if (!editor || editor.document.languageId !== 'quarkup') {
        vscode.window.showWarningMessage('Open a Quarkup (.qu) file first.');
        return;
    }

    const key = editor.document.uri.toString();
    const existing = panelsByDocument.get(key);
    if (existing) {
        existing.reveal(column);
        return;
    }

    const assetsRoot = vscode.Uri.joinPath(context.extensionUri, 'media', 'pkg', 'quarkup.js');
    if (!fs.existsSync(assetsRoot.fsPath)) {
        vscode.window.showErrorMessage(
            'Quarkup preview assets are missing. Run "npm run build-assets" in the extension directory ' +
                '(requires the wasm32-unknown-unknown Rust target and a matching wasm-bindgen-cli).'
        );
        return;
    }

    const panel = new QuarkupPreviewPanel(context, editor.document, column);
    panelsByDocument.set(key, panel);
    panel.onDidDispose(() => panelsByDocument.delete(key));
}

class QuarkupPreviewPanel {
    private readonly panel: vscode.WebviewPanel;
    private readonly document: vscode.TextDocument;
    private readonly disposables: vscode.Disposable[] = [];
    private debounceTimer: ReturnType<typeof setTimeout> | undefined;
    private webviewReady = false;

    constructor(context: vscode.ExtensionContext, document: vscode.TextDocument, column: vscode.ViewColumn) {
        this.document = document;

        this.panel = vscode.window.createWebviewPanel(
            'quarkupPreview',
            `Preview: ${path.basename(document.fileName)}`,
            column,
            {
                enableScripts: true,
                localResourceRoots: [vscode.Uri.joinPath(context.extensionUri, 'media')],
                retainContextWhenHidden: true,
            }
        );

        this.panel.webview.html = getWebviewHtml(this.panel.webview, context.extensionUri);

        this.disposables.push(
            this.panel.webview.onDidReceiveMessage((message: { type: string }) => {
                if (message.type === 'ready') {
                    this.webviewReady = true;
                    this.pushSource(this.document.getText());
                }
            }),
            vscode.workspace.onDidChangeTextDocument((event) => {
                if (event.document.uri.toString() === this.document.uri.toString()) {
                    this.scheduleUpdate(event.document.getText());
                }
            }),
            vscode.workspace.onDidCloseTextDocument((closed) => {
                if (closed.uri.toString() === this.document.uri.toString()) {
                    this.panel.dispose();
                }
            })
        );

        this.panel.onDidDispose(() => this.cleanup(), null, this.disposables);
    }

    reveal(column: vscode.ViewColumn): void {
        this.panel.reveal(column);
    }

    onDidDispose(listener: () => void): void {
        this.panel.onDidDispose(listener);
    }

    dispose(): void {
        this.panel.dispose();
    }

    private scheduleUpdate(source: string): void {
        clearTimeout(this.debounceTimer);
        this.debounceTimer = setTimeout(() => this.pushSource(source), DEBOUNCE_MS);
    }

    private pushSource(source: string): void {
        if (!this.webviewReady) {
            return;
        }
        this.panel.webview.postMessage({ type: 'update', source });
    }

    private cleanup(): void {
        clearTimeout(this.debounceTimer);
        for (const d of this.disposables) {
            d.dispose();
        }
    }
}

function getNonce(): string {
    const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    let text = '';
    for (let i = 0; i < 32; i++) {
        text += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return text;
}

function getKatexCss(webview: vscode.Webview, extensionUri: vscode.Uri): string {
    const cssPath = vscode.Uri.joinPath(extensionUri, 'media', 'vendor', 'katex', 'katex.min.css');
    const fontsUri = webview.asWebviewUri(vscode.Uri.joinPath(extensionUri, 'media', 'vendor', 'katex', 'fonts'));
    const raw = fs.readFileSync(cssPath.fsPath, 'utf8');
    // Rewrite katex.min.css's relative `url(fonts/...)` references to absolute
    // webview URIs. The CSS is injected as an inline <style> (see renderMath
    // in the webview script below) rather than a dynamically-appended <link>,
    // because a <link> added to the sandboxed preview iframe after the fact
    // has no CSP nonce and was silently failing to load under the webview's
    // stricter CSP — unlike the plain browser build, which has no CSP at all.
    return raw.replace(/url\(fonts\//g, `url(${fontsUri.toString()}/`);
}

function getWebviewHtml(webview: vscode.Webview, extensionUri: vscode.Uri): string {
    const nonce = getNonce();
    const pkgJsUri = webview.asWebviewUri(vscode.Uri.joinPath(extensionUri, 'media', 'pkg', 'quarkup.js'));
    const katexCss = getKatexCss(webview, extensionUri);

    // The compiled Quarkup document (loaded into the sandboxed preview
    // iframe below) carries its own <style> block — from the default
    // template or a user-supplied --template — that can't be known ahead of
    // time and so can't be nonce'd. srcdoc content inherits this same CSP,
    // and CSP from multiple sources is always additive (most-restrictive
    // wins), so a permissive <meta> tag inside that content wouldn't help.
    // style-src/img-src/font-src are relaxed with 'unsafe-inline' instead;
    // this is safe because the iframe's `sandbox` attribute (no
    // allow-scripts) already blocks script execution regardless of CSP, so
    // there's no script to inject via CSS/HTML in the first place. Only
    // script-src stays nonce-gated, since it governs this outer page.
    const csp = [
        "default-src 'none'",
        `img-src ${webview.cspSource} https: data:`,
        `style-src ${webview.cspSource} 'unsafe-inline'`,
        `font-src ${webview.cspSource} data:`,
        `connect-src ${webview.cspSource}`,
        `script-src 'nonce-${nonce}' 'wasm-unsafe-eval'`,
    ].join('; ');

    return `<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta http-equiv="Content-Security-Policy" content="${csp}">
<style nonce="${nonce}">
    html, body {
        height: 100%;
        margin: 0;
        padding: 0;
        background: var(--vscode-editor-background);
    }
    #preview {
        display: block;
        width: 100%;
        height: 100vh;
        border: none;
        background: white;
    }
    #error {
        display: none;
        padding: 1rem;
        font-family: var(--vscode-editor-font-family, monospace);
        color: var(--vscode-errorForeground);
        white-space: pre-wrap;
    }
</style>
</head>
<body>
<div id="error"></div>
<iframe id="preview" title="Quarkup preview" sandbox="allow-same-origin"></iframe>
<script nonce="${nonce}" defer src="${webview
        .asWebviewUri(vscode.Uri.joinPath(extensionUri, 'media', 'vendor', 'katex', 'katex.min.js'))
        .toString()}"></script>
<script type="module" nonce="${nonce}">
    import init, { compile } from "${pkgJsUri}";

    const vscode = acquireVsCodeApi();
    const previewEl = document.getElementById('preview');
    const errorEl = document.getElementById('error');

    let compileFn = null;
    let latestSource = '';

    const katexStyleTag = '<style>' + ${JSON.stringify(katexCss)} + '</style>';

    // The wasm build can't run mathjax-svg-rs (it needs OS threads), so math
    // is emitted as .quarkup-math-tex placeholders carrying raw LaTeX,
    // typeset here with KaTeX once the preview iframe has loaded. The KaTeX
    // CSS is baked into the document *before* it's assigned to srcdoc (see
    // runCompile below) rather than appended to the iframe afterwards — a
    // <link>/<style> injected post-load into the sandboxed iframe has no
    // matching CSP nonce there and was silently getting blocked.
    function renderMath() {
        const doc = previewEl.contentDocument;
        if (!doc || typeof katex === 'undefined') return;

        doc.querySelectorAll('.quarkup-math-tex').forEach((el) => {
            const tex = el.getAttribute('data-latex') || '';
            const displayMode = el.getAttribute('data-display') === 'true';
            try {
                el.innerHTML = katex.renderToString(tex, { throwOnError: false, displayMode });
            } catch (err) {
                // leave the raw LaTeX source visible as a fallback
            }
        });
    }
    previewEl.addEventListener('load', renderMath);

    function runCompile() {
        if (!compileFn) return;
        try {
            const html = compileFn(latestSource, undefined, false, '');
            const htmlWithKatexCss = html.includes('</head>')
                ? html.replace('</head>', katexStyleTag + '</head>')
                : katexStyleTag + html;
            errorEl.style.display = 'none';
            previewEl.style.display = 'block';
            previewEl.srcdoc = htmlWithKatexCss;
        } catch (err) {
            previewEl.style.display = 'none';
            errorEl.style.display = 'block';
            errorEl.textContent = 'Quarkup compile error: ' + err;
        }
    }

    window.addEventListener('message', (event) => {
        const message = event.data;
        if (message.type === 'update') {
            latestSource = message.source;
            runCompile();
        }
    });

    init()
        .then(() => {
            compileFn = compile;
            vscode.postMessage({ type: 'ready' });
        })
        .catch((err) => {
            errorEl.style.display = 'block';
            errorEl.textContent = 'Failed to load the Quarkup compiler: ' + err;
        });
</script>
</body>
</html>`;
}
