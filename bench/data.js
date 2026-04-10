window.BENCHMARK_DATA = {
  "lastUpdate": 1775854416134,
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
      }
    ]
  }
}