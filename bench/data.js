window.BENCHMARK_DATA = {
  "lastUpdate": 1775948774559,
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
          "id": "aaa7c7c358abe2294951b1d80a295afbb5323cae",
          "message": "Merge pull request #2 from jamtur01/fix/version-bump\n\nBump to v0.1.1",
          "timestamp": "2026-04-10T17:24:44-04:00",
          "tree_id": "9b1b6428b797df0aa6679f5df2a379df67c72f0f",
          "url": "https://github.com/jamtur01/decruft/commit/aaa7c7c358abe2294951b1d80a295afbb5323cae"
        },
        "date": 1775856425051,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2465132,
            "range": "± 65795",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3418525,
            "range": "± 71387",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 368927047,
            "range": "± 4835788",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 3814339,
            "range": "± 72090",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2634160,
            "range": "± 37766",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 239900177,
            "range": "± 5478280",
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
          "id": "6772b8b9ebb0ecee3e4988289bfb5063fbafd0ae",
          "message": "Merge pull request #3 from jamtur01/fix/msrv\n\nAdd MSRV 1.85",
          "timestamp": "2026-04-11T12:00:22-04:00",
          "tree_id": "b60e7a899f83733913b7c2ec35ca7debfd1531fe",
          "url": "https://github.com/jamtur01/decruft/commit/6772b8b9ebb0ecee3e4988289bfb5063fbafd0ae"
        },
        "date": 1775923365551,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2620783,
            "range": "± 21959",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3671913,
            "range": "± 68220",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 387532507,
            "range": "± 4302566",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 3923975,
            "range": "± 41147",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2815840,
            "range": "± 11939",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 243970089,
            "range": "± 2836851",
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
          "id": "c57021c194155a9cdbab7f957ea6697a46eff321",
          "message": "Merge pull request #4 from jamtur01/feat/dublin-core\n\nfeat(metadata): add Dublin Core and Parsely metadata support",
          "timestamp": "2026-04-11T12:58:06-04:00",
          "tree_id": "1b8a0b24a6b1692762bddbb4ff8c49bbfe89a98f",
          "url": "https://github.com/jamtur01/decruft/commit/c57021c194155a9cdbab7f957ea6697a46eff321"
        },
        "date": 1775926829341,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2702983,
            "range": "± 21279",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3743987,
            "range": "± 31484",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 388042430,
            "range": "± 946443",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 4118214,
            "range": "± 26369",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2875842,
            "range": "± 75021",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 244174725,
            "range": "± 1299088",
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
          "id": "9a056a4cbea53138ce503a873d0f18fba0539458",
          "message": "Merge pull request #9 from jamtur01/feat/oracle-fixtures\n\ntest: add oracle fixture tests against defuddle output",
          "timestamp": "2026-04-11T13:28:21-04:00",
          "tree_id": "e1fe45f2e8f01a9a227a55f046c2968af0bb2ee2",
          "url": "https://github.com/jamtur01/decruft/commit/9a056a4cbea53138ce503a873d0f18fba0539458"
        },
        "date": 1775928647901,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2677503,
            "range": "± 26722",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3681506,
            "range": "± 27522",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 392078907,
            "range": "± 5579012",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 4197670,
            "range": "± 28858",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2846726,
            "range": "± 90785",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 241951680,
            "range": "± 2124518",
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
          "id": "17791f42412338c1cd05b69dd107be37b530b84c",
          "message": "Merge pull request #15 from jamtur01/fix/content-removal-bugs\n\nFix content removal bugs (#7, #8, #10, #13)",
          "timestamp": "2026-04-11T14:05:01-04:00",
          "tree_id": "57c6abb95ae7e4d9d0acf546e239dd1d5a0c41c9",
          "url": "https://github.com/jamtur01/decruft/commit/17791f42412338c1cd05b69dd107be37b530b84c"
        },
        "date": 1775930845094,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2724976,
            "range": "± 16038",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3798843,
            "range": "± 18800",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 397507022,
            "range": "± 5842868",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 4150118,
            "range": "± 80516",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2951042,
            "range": "± 29093",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 250679375,
            "range": "± 2230874",
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
          "id": "f042670a308f9462f0e11165bf0ab98de8f891bd",
          "message": "Merge pull request #16 from jamtur01/feat/golden-file-tests\n\nRationalize test suite, fix metadata extraction",
          "timestamp": "2026-04-11T18:04:01-04:00",
          "tree_id": "d8a08e6f2880106df41249825c731d7d316955cb",
          "url": "https://github.com/jamtur01/decruft/commit/f042670a308f9462f0e11165bf0ab98de8f891bd"
        },
        "date": 1775945195629,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2755122,
            "range": "± 12546",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3822703,
            "range": "± 222845",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 399994648,
            "range": "± 2053118",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 4159910,
            "range": "± 26725",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2960298,
            "range": "± 55945",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 251846660,
            "range": "± 10872024",
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
          "id": "7225b71d434181221fff46f823c0005b721b75e2",
          "message": "Merge pull request #20 from jamtur01/fix/published-dates-and-salon\n\nFix published date extraction, close salon-1",
          "timestamp": "2026-04-11T18:54:36-04:00",
          "tree_id": "22a3468bad3bd70c17ad8e6d46128595ae63a458",
          "url": "https://github.com/jamtur01/decruft/commit/7225b71d434181221fff46f823c0005b721b75e2"
        },
        "date": 1775948230189,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2718041,
            "range": "± 29606",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3770710,
            "range": "± 74805",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 402556938,
            "range": "± 6243758",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 4175487,
            "range": "± 19591",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2859336,
            "range": "± 12726",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 247183469,
            "range": "± 1017934",
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
          "id": "f37fb83e94b54c91e6b6532f6fb30074e39451f6",
          "message": "feat: 3-way comparison script (decruft vs defuddle vs readabilityrs)\n\nAdd readabilityrs as third column in compare_sites.sh. Includes a\nminimal CLI wrapper in tools/readabilityrs-cli/ that builds from the\npublished crate.\n\nIssues are gated on decruft vs defuddle only — readabilityrs word\ncounts are shown for reference but don't flag failures (it extracts\n3-7x more words than both decruft and defuddle on most pages).",
          "timestamp": "2026-04-11T19:01:47-04:00",
          "tree_id": "e7f6b7125e4a192bcfe2f6ea2c01006e8f029079",
          "url": "https://github.com/jamtur01/decruft/commit/f37fb83e94b54c91e6b6532f6fb30074e39451f6"
        },
        "date": 1775948670633,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2946257,
            "range": "± 47300",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3865855,
            "range": "± 181368",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 417276278,
            "range": "± 14411448",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 4225115,
            "range": "± 48614",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 3182916,
            "range": "± 81421",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 264124298,
            "range": "± 3039625",
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
          "id": "7225b71d434181221fff46f823c0005b721b75e2",
          "message": "Merge pull request #20 from jamtur01/fix/published-dates-and-salon\n\nFix published date extraction, close salon-1",
          "timestamp": "2026-04-11T18:54:36-04:00",
          "tree_id": "22a3468bad3bd70c17ad8e6d46128595ae63a458",
          "url": "https://github.com/jamtur01/decruft/commit/7225b71d434181221fff46f823c0005b721b75e2"
        },
        "date": 1775948773830,
        "tool": "cargo",
        "benches": [
          {
            "name": "small_page (12KB blog)",
            "value": 2592668,
            "range": "± 95838",
            "unit": "ns/iter"
          },
          {
            "name": "medium_page (317KB stephango)",
            "value": 3626295,
            "range": "± 58636",
            "unit": "ns/iter"
          },
          {
            "name": "large_page (1.1MB wikipedia)",
            "value": 382373899,
            "range": "± 4756142",
            "unit": "ns/iter"
          },
          {
            "name": "github_issue (267KB)",
            "value": 4003426,
            "range": "± 84916",
            "unit": "ns/iter"
          },
          {
            "name": "markdown_output (12KB blog)",
            "value": 2775037,
            "range": "± 12771",
            "unit": "ns/iter"
          },
          {
            "name": "large_page_no_scoring (1.1MB)",
            "value": 249082680,
            "range": "± 1388771",
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}