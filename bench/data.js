window.BENCHMARK_DATA = {
  "lastUpdate": 1775855954964,
  "repoUrl": "https://github.com/jamtur01/decruft",
  "entries": {
    "decruft benchmarks": [
      {
        "commit": {
          "author": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "committer": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "distinct": true,
          "id": "8d01ee963ef5aab0ddefa95f4fe7531b4e3584cb",
          "message": "Add CI benchmark workflow with regression detection",
          "timestamp": "2026-04-10T16:50:06-04:00",
          "tree_id": "18623eae18947c0a466025de2ec36c4e31b90356",
          "url": "https://github.com/jamtur01/decruft/commit/8d01ee963ef5aab0ddefa95f4fe7531b4e3584cb"
        },
        "date": 1775854415776,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2623732,
            "range": "± 19771",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3637783,
            "range": "± 24410",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 396762898,
            "range": "± 10864777",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 4028034,
            "range": "± 47635",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2760424,
            "range": "± 146950",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 242631422,
            "range": "± 3685568",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "committer": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "distinct": true,
          "id": "7dc20fa083733eae8fe3e13f93e5ef9c3c2f8d6c",
          "message": "Fix bench workflow: store data in gh-pages/bench/, no Pages conflict",
          "timestamp": "2026-04-10T16:52:17-04:00",
          "tree_id": "9d535bd6370f034df82fc7391ce598b4b419c449",
          "url": "https://github.com/jamtur01/decruft/commit/7dc20fa083733eae8fe3e13f93e5ef9c3c2f8d6c"
        },
        "date": 1775854537286,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2601601,
            "range": "± 9488",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3659294,
            "range": "± 13814",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 385049305,
            "range": "± 2797464",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 3882867,
            "range": "± 26187",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2778455,
            "range": "± 8750",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 240637450,
            "range": "± 1884481",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "committer": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "distinct": true,
          "id": "df8aaf278afda0ba09d857329c633a9485c9d4ed",
          "message": "Update actions/checkout to v5 (Node.js 24 support)",
          "timestamp": "2026-04-10T16:56:00-04:00",
          "tree_id": "c7a3807bf26094820c877647f46c70eb8f98f533",
          "url": "https://github.com/jamtur01/decruft/commit/df8aaf278afda0ba09d857329c633a9485c9d4ed"
        },
        "date": 1775854704497,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2584233,
            "range": "± 60661",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3587363,
            "range": "± 22252",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 386959637,
            "range": "± 8622479",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 3946416,
            "range": "± 30394",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2769021,
            "range": "± 17130",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 241070821,
            "range": "± 1748229",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "committer": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "distinct": true,
          "id": "6448ce28d9e745f0d2b0cce33813980fc5cd3f96",
          "message": "Update all GitHub Actions to latest versions (Node.js 24)\n\n- actions/configure-pages v5 -> v6\n- actions/deploy-pages v4 -> v5\n- actions/upload-artifact v4 -> v7\n- actions/download-artifact v4 -> v8\n- actions/upload-pages-artifact v3 -> v5\n- softprops/action-gh-release v2 -> v2.6.1\n- Swatinem/rust-cache v2 -> v2.9.1\n- benchmark-action/github-action-benchmark v1 -> v1.22.0",
          "timestamp": "2026-04-10T16:58:21-04:00",
          "tree_id": "c3443ec445e4bd31a972f82d9e4fbd30ad5fcd8b",
          "url": "https://github.com/jamtur01/decruft/commit/6448ce28d9e745f0d2b0cce33813980fc5cd3f96"
        },
        "date": 1775854848273,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2586245,
            "range": "± 44024",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3577156,
            "range": "± 18548",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 391290193,
            "range": "± 5666495",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 3919343,
            "range": "± 16454",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2751187,
            "range": "± 18176",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 240283410,
            "range": "± 2462799",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "committer": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "distinct": true,
          "id": "3fe007e3b619c1d80c43fdbd68e94b6d73546e3b",
          "message": "Fix upload-pages-artifact version tag (v5 -> v5.0.0)",
          "timestamp": "2026-04-10T16:59:41-04:00",
          "tree_id": "34e00a40524f5d20ea3477f4ab58a216fb84c186",
          "url": "https://github.com/jamtur01/decruft/commit/3fe007e3b619c1d80c43fdbd68e94b6d73546e3b"
        },
        "date": 1775854929613,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2613967,
            "range": "± 14371",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3658437,
            "range": "± 58983",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 387807851,
            "range": "± 2053564",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 3907713,
            "range": "± 26935",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2800200,
            "range": "± 27376",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 242215013,
            "range": "± 863866",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "committer": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "distinct": true,
          "id": "1421757cd3add7cd9b27b931e1e56acdbb508bd6",
          "message": "Add permissions: contents: read to CI workflow (CodeQL alert #2)",
          "timestamp": "2026-04-10T17:01:17-04:00",
          "tree_id": "c8ec0447e2f926255d63cf4d9ea2253e7c9cf728",
          "url": "https://github.com/jamtur01/decruft/commit/1421757cd3add7cd9b27b931e1e56acdbb508bd6"
        },
        "date": 1775855019710,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2585166,
            "range": "± 12625",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3588371,
            "range": "± 106631",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 391398126,
            "range": "± 5253189",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 3921631,
            "range": "± 26000",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2751325,
            "range": "± 12075",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 240210645,
            "range": "± 1456076",
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "james@lovedthanlost.net",
            "name": "James Turnbull",
            "username": "jamtur01"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "01c83720ba1e7efaa26c52e905e8aa8cc27b42e5",
          "message": "Merge pull request #1 from jamtur01/fix/restore-readme\n\nRestore README.md",
          "timestamp": "2026-04-10T17:16:38-04:00",
          "tree_id": "2c394b29b39be25fb702b4d31db5209eb4ff9388",
          "url": "https://github.com/jamtur01/decruft/commit/01c83720ba1e7efaa26c52e905e8aa8cc27b42e5"
        },
        "date": 1775855953935,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2569262,
            "range": "± 22926",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3593902,
            "range": "± 40809",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 387882810,
            "range": "± 6347498",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 3947963,
            "range": "± 61547",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2731184,
            "range": "± 35641",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 243740716,
            "range": "± 9415569",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}