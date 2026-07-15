// Ad-hoc verification harness: tokenizes sample Quarkup snippets with the
// same libraries VS Code uses internally (vscode-textmate + vscode-oniguruma),
// and prints the resulting scopes so grammar changes can be eyeballed without
// a running VS Code instance. Not part of the shipped extension.
const fs = require('fs');
const path = require('path');
const oniguruma = require('vscode-oniguruma');
const { Registry, parseRawGrammar } = require('vscode-textmate');

const wasmPath = require.resolve('vscode-oniguruma/release/onig.wasm');

async function main() {
    const wasmBin = fs.readFileSync(wasmPath).buffer;
    await oniguruma.loadWASM(wasmBin);

    const onigLib = Promise.resolve({
        createOnigScanner: (patterns) => new oniguruma.OnigScanner(patterns),
        createOnigString: (s) => new oniguruma.OnigString(s),
    });

    const grammarPath = path.join(__dirname, '..', 'syntaxes', 'quarkup.tmLanguage.json');

    const registry = new Registry({
        onigLib,
        loadGrammar: async (scopeName) => {
            if (scopeName === 'source.quarkup') {
                const content = fs.readFileSync(grammarPath, 'utf8');
                return parseRawGrammar(content, grammarPath);
            }
            // Embedded language grammars (source.rust, source.js, ...) aren't
            // available outside a real VS Code install; returning null just
            // means that content stays untokenized within its contentName
            // scope, which is fine for verifying the Quarkup grammar itself.
            return null;
        },
    });

    const grammar = await registry.loadGrammar('source.quarkup');

    const samples = fs.readFileSync(path.join(__dirname, 'sample.qu'), 'utf8').split('\n');

    let ruleStack = oniguruma.INITIAL ?? undefined;
    const vsctm = require('vscode-textmate');
    let stack = vsctm.INITIAL;

    let failures = 0;
    for (const line of samples) {
        const result = grammar.tokenizeLine(line, stack);
        stack = result.ruleStack;
        console.log(JSON.stringify(line));
        for (const token of result.tokens) {
            const text = line.substring(token.startIndex, token.endIndex);
            console.log(`  ${JSON.stringify(text)} -> ${token.scopes.join(' ')}`);
        }
    }
    console.log(failures === 0 ? '\nOK' : `\n${failures} issue(s) found`);
}

main().catch((err) => {
    console.error(err);
    process.exit(1);
});
