#!/usr/bin/env bash
set -euo pipefail

# Compare decruft vs defuddle output across diverse site types
# Usage: ./tests/compare_sites.sh [--fix]

DECRUFT="./target/release/decruft"
OUTDIR="/tmp/decruft-compare"
mkdir -p "$OUTDIR"

cargo build --release 2>/dev/null

URLS=(
    # News
    "https://www.bbc.com/news/articles/cx2k7r2nge3o"
    # Blog
    "https://www.paulgraham.com/read.html"
    # Tech blog
    "https://blog.rust-lang.org/2024/11/28/Rust-2024-Edition.html"
    # Wikipedia
    "https://en.wikipedia.org/wiki/Rust_(programming_language)"
    # GitHub issue
    "https://github.com/kepano/defuddle/issues/56"
    # Documentation
    "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html"
    # Medium-style
    "https://stackoverflow.blog/2024/12/04/the-developer-who-saw-the-matrix/"
    # Academic
    "https://arxiv.org/abs/2401.00001"
    # Substack
    "https://www.lennysnewsletter.com/p/how-to-build-a-billion-dollar-ai"
)

PASS=0
FAIL=0
WARN=0

for url in "${URLS[@]}"; do
    name=$(echo "$url" | sed 's|https\?://||;s|/|_|g;s|[^a-zA-Z0-9_.-]||g' | head -c 80)
    echo "=== $name ==="
    echo "    URL: $url"

    # Fetch HTML once
    html_file="$OUTDIR/${name}.html"
    if [ ! -f "$html_file" ]; then
        curl -sL --max-time 15 -o "$html_file" "$url" 2>/dev/null || {
            echo "    SKIP: fetch failed"
            continue
        }
    fi

    # Run decruft
    dc_json="$OUTDIR/${name}.decruft.json"
    $DECRUFT "$html_file" --url "$url" -f json > "$dc_json" 2>/dev/null || {
        echo "    FAIL: decruft crashed"
        FAIL=$((FAIL + 1))
        continue
    }

    # Run defuddle
    df_json="$OUTDIR/${name}.defuddle.json"
    echo "<html>$(cat "$html_file")</html>" | npx defuddle parse --json /dev/stdin 2>/dev/null > "$df_json" || {
        # Try file path instead
        npx defuddle parse --json "$html_file" 2>/dev/null > "$df_json" || {
            echo "    SKIP: defuddle failed"
            continue
        }
    }

    # Compare
    python3 - "$dc_json" "$df_json" "$name" << 'PYEOF'
import json, sys, re

dc_file, df_file, name = sys.argv[1], sys.argv[2], sys.argv[3]

with open(dc_file) as f:
    dc = json.load(f)
with open(df_file) as f:
    df = json.load(f)

issues = []

# Title
if dc.get('title', '') != df.get('title', ''):
    issues.append(f"title: '{dc.get('title','')}' vs '{df.get('title','')}'")

# Word count ratio
dc_wc = dc.get('word_count', 0)
df_wc = df.get('wordCount', 0)
if df_wc > 0:
    ratio = dc_wc / df_wc
    if ratio < 0.5 or ratio > 2.0:
        issues.append(f"word_count: {dc_wc} vs {df_wc} ({ratio:.1f}x)")

# Author
dc_author = dc.get('author', '')
df_author = df.get('author', '')
if dc_author != df_author and not (dc_author in df_author or df_author in dc_author):
    issues.append(f"author: '{dc_author}' vs '{df_author}'")

# Language
if dc.get('language', '') != df.get('language', ''):
    issues.append(f"language: '{dc.get('language','')}' vs '{df.get('language','')}'")

# Internal attributes leaking
if 'data-decruft-' in dc.get('content', ''):
    count = dc['content'].count('data-decruft-')
    issues.append(f"leaked {count} data-decruft attributes")

# Content sanity
if dc_wc == 0 and df_wc > 0:
    issues.append("EMPTY: decruft extracted nothing")

if issues:
    print(f"    ISSUES ({len(issues)}):")
    for i in issues:
        print(f"      - {i}")
else:
    print(f"    OK: title='{dc.get('title','')}' words={dc_wc} (defuddle={df_wc})")

sys.exit(1 if any('EMPTY' in i for i in issues) else 0)
PYEOF

    result=$?
    if [ $result -eq 0 ]; then
        PASS=$((PASS + 1))
    else
        WARN=$((WARN + 1))
    fi
    echo ""
done

echo "============================================"
echo "Results: $PASS pass, $WARN with issues, $FAIL failed"
echo "============================================"
