# Roadmap v2 progress

Written only by the coordinator. Rules: `00-run-the-pass.md` §0a "Progress file".
Status: `todo`, `running`, `done`, `blocked`, `skipped`.

| WP | Wave | Status | Branch | PR | Notes |
|---|---|---|---|---|---|
| WP-00 | 0 | done | - | #201 | closes #183 |
| WP-01a | 1 | done | - | #202 |  |
| WP-01b | 1 | done | - | #204 |  |
| WP-01c | 1 | done | - | #206 |  |
| WP-01d | 1 | done | - | #207 |  |
| WP-02 | 1 | done | - | #203 |  |
| WP-25a | 1 | done | - | #205 |  |
| WP-03 | 2 | done | - | #243 | closes #76 |
| WP-04 | 2 | done | - | #249 | closes #116 |
| WP-05 | 2 | done | - | #244 | closes #118 |
| WP-06 | 2 | done | - | #247 |  |
| WP-07 | 2 | done | - | #245 |  |
| WP-11 | 2 | done | - | #248 | follow-up: Healer holds heals after a backward clock step; lapses with WP-36 |
| WP-12 | 2 | done | - | #246 |  |
| WP-13 | 2 | done | - | #215 |  |
| WP-14 | 2 | done | - | #217 | follow-up: apply_with never installs the fetched profile and replaces an existing config.txt; fixed in #278 (closes #275), which installs the profile and merges our keys into its config.txt |
| WP-16 | 2 | done | - | #250 | follow-up: fixed in #307 |
| WP-17 | 2 | done | - | #218 |  |
| WP-18 | 2 | done | - | #220 |  |
| WP-20 | 2 | done | - | #219 |  |
| WP-21 | 2 | done | - | #221 |  |
| WP-22 | 2 | done | - | #222 |  |
| WP-23 | 2 | done | - | #230 | closes #117 |
| WP-24 | 2 | done | - | #231 |  |
| WP-25b | 2 | done | - | #252 |  |
| WP-35 | 2b | done | - | #279 | closes #224 |
| WP-36 | 2b | running | - | #284 | first PR merged (closes #225): the tether is out of watch, exec, api and settings; the gnirehtet left in install.rs and cli/link.rs is listed for #226 |
| WP-37 | 2b | todo | - | - | |
| WP-08 | 3 | done | - | #233 |  |
| WP-10 | 3 | done | - | #253 | superseded by ruling 42; WP-36 removes its code |
| WP-15 | 3 | done | - | #236 |  |
| WP-19 | 3 | done | - | #234 |  |
| WP-26 | 3 | done | - | #237 |  |
| WP-27 | 3 | done | - | #268 | follow-up: undo leaves empty folders the apply step made (record format) |
| WP-28 | 3 | done | - | #267 | closes #121, #75; merged before WP-36 |
| WP-30 | 3 | todo | - | - | |
| WP-09 | 4 | todo | - | - | |
| WP-29 | 4 | todo | - | - | |
| WP-31 | 4 | todo | - | - | |
| WP-32 | 4 | done | - | #251 | merged before WP-28 and WP-37 |
| WP-33 | 4 | done | - | #272 | merged before WP-37 |
| WP-38 | 5 | todo | - | - | |
| WP-34 | 6 | todo | - | - | |

Brought up to date from `git log origin/main` and `gh pr list --state merged` at `d257671` (2026-11-12). Merged since `3e5ea15` and outside the WPs: #278 (#275), #280 (#223), #281 (#38), #282 (#37). The WPs were merged one PR at a time into `main` (§0), not through `v2/integration`, so the Branch column stays `-`.

Run log (one line per stop: time, reason, tip of `v2/integration`):
