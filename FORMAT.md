# Code Formatting

## Create `.oxfmtrc.json`

```bash
cat > .oxfmtrc.json <<'EOF'
{
    "printWidth": 120,
    "tabWidth": 4,
    "useTabs": false,
    "bracketSameLine": true,
    "htmlWhitespaceSensitivity": "ignore",
    "embeddedLanguageFormatting": "auto",
    "ignorePatterns": [
        "node_modules/**",
        "dist/**",
        ".output/**"
    ]
}
EOF
```

## Check formatting without modifying files

```bash
npx oxfmt@latest "**/*.{vue,scss}" --check
```

## Format Vue and SCSS files

```bash
npx oxfmt@latest "**/*.{vue,scss}" --write
```
