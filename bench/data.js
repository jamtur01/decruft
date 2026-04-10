window.BENCHMARK_DATA = {
  "lastUpdate": 1775854704849,
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
      }
    ]
  }
}