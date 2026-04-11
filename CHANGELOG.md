## v0.1.3

- feat: release 0.1.3
- build: exclude test fixtures from published crate
- Update CHANGELOG for v0.1.2
# Changelog

## v0.1.2

- feat: release 0.1.2
- build: add release script
- Merge pull request #20 from jamtur01/fix/published-dates-and-salon
- fix: validate ISO dates properly, accept 1900s years
- fix: extract published dates from text elements (#17)
- Merge pull request #16 from jamtur01/feat/golden-file-tests
- fix: address PR review, update docs
- fix: byte-exact golden comparison, pass fixture URLs
- fix: zero tolerance in all tests, add golden markdown
- refactor: consolidate fixtures into single flat directory
- fix: x/twitter extractor and github title cleanup
- fix: spurious author from comments, GitHub site name, extractor override
- fix: extract GitHub username from URL, strip URLs from author
- fix: infer site name from title suffix, strip accordingly
- fix: use author as site name fallback (117/144 metadata match)
- test: exact upstream oracle with per-fixture pass lists
- test: upstream oracles, markdown coverage, hard fixtures
- test: un-ignore network tests, add fix regression tests
- refactor: rationalize test suite into 6 focused files
- test: add golden file regression tests
- Merge pull request #15 from jamtur01/fix/content-removal-bugs
- fix: content removal bugs (#7, #8, #10, #13)
- Merge pull request #9 from jamtur01/feat/oracle-fixtures
- Merge branch 'main' into feat/oracle-fixtures
- test: add issue references to oracle test thresholds
- test: add markdown quality test suite
- test: tighten oracle and mozilla suite thresholds
- Merge pull request #4 from jamtur01/feat/dublin-core
- test: add Mozilla Readability test suite (130 fixtures)
- test: add oracle fixture tests against defuddle output
- feat(metadata): add canonical_url, keywords, content_type
- feat(metadata): add modified time, DC.publisher, expanded DCTERMS
- feat(metadata): add Dublin Core and Parsely metadata support
- Merge pull request #3 from jamtur01/fix/msrv
- build: add MSRV 1.85 and conventional commit hook
- Add MSRV 1.85 (minimum for edition 2024)
- Merge pull request #2 from jamtur01/fix/version-bump
