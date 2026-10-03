# embedded-alerts/eal-mash-web#11 — docs: record Embedded Alerts web/API data paths

head: agent/web-api-four-path-1399  base: main  author: ORESoftware  updated: 2026-09-04T21:53:20Z
dir: /Users/maca5/codes/.claude-fleet/scratch/merge/embedded-alerts_eal-mash-web__11

## conflicted files
- docs/web-api-data-access.md
- src/main.rs

## base (main) last 8 commits
1891021 Merge pull request #17 from embedded-alerts/salvage/DEN-3960-embedded-web-api-paths
62e292b DEN-3960: install pinned ores-sops in CI
37e6d70 DEN-3960: reconcile Embedded Alerts data paths
a2089f0 Merge pull request #13 from embedded-alerts/feature/commit-embeddings-20260824
67fe8af Merge pull request #14 from embedded-alerts/agent/ores-sops-ensure-dec-20260828b
9a90205 Refuse unguarded env/dec mkdir before ores-sops.
5e0ff4b Merge pull request #9 from embedded-alerts/agent/den-3461-canonical-operator-console
2ea2513 merge main into canonical operator console

## head (agent/web-api-four-path-1399) last 8 commits
dae6265 docs: record Embedded Alerts web API data paths
b60092b Merge branch 'main' of github.com:embedded-alerts/eal-mash-web
4baa898 chore: ignore tmp/temp worktree scratch directories
952e04e Merge remote:agent/zed-dependency-graph into main with canonical policy reconciliation
086f452 Prefer primary branches and avoid agent worktrees
5942b99 Restore the Embedded Alerts MASH dashboard
6be538a align Zed install directory with package family
f9566c2 Wire MASH runtime dependencies through Zed

## merge-base: b60092b4c42cd49e739e4bb938c87b65f28ace30

## PR diff stat (merge-base..head)
 docs/web-api-data-access.md | 97 +++++++++++++++++++++++++++++++++++++++++++++
 src/main.rs                 |  4 ++
 2 files changed, 101 insertions(+)

## base diff stat (merge-base..base)
 .nix/README.md                              |   6 +
 .nix/flake.nix                              |  55 ++
 .sops.yaml                                  |  58 ++
 Cargo.toml                                  |  14 +-
 Dockerfile                                  |  15 +-
 README.md                                   |  70 ++-
 docs/architecture.md                        |  44 +-
 docs/web-api-data-access.md                 | 104 ++++
 env/README.md                               | 158 +++++
 env/enc/dev.env.enc                         |  18 +
 env/enc/prod.env.enc                        |  18 +
 justfile                                    |  57 ++
 operator-console/.env.example               |   8 +
 operator-console/Cargo.toml                 |  21 +
 operator-console/README.md                  |  44 ++
 operator-console/scripts/verify_contract.py |  40 ++
 operator-console/src/api.rs                 | 250 ++++++++
 operator-console/src/config.rs              | 152 +++++
 operator-console/src/main.rs                | 198 ++++++
 operator-console/src/models.rs              | 190 ++++++
 operator-console/src/views.rs               | 277 +++++++++
 scripts/sops-entrypoint.sh                  |  62 ++
 shell                                       |  11 +
 src/api.rs                                  | 257 ++++++++
 src/config.rs                               | 164 +++++
 src/main.rs                                 | 147 ++---
 src/models.rs                               | 423 +++++++++++++
 src/routes.rs                               | 179 ++++++
 src/views.rs                                | 900 ++++++++++++++++++++++++++++
 40 files changed, 4542 insertions(+), 152 deletions(-)

## merge output
Auto-merging docs/web-api-data-access.md
CONFLICT (add/add): Merge conflict in docs/web-api-data-access.md
Auto-merging src/main.rs
CONFLICT (content): Merge conflict in src/main.rs
Automatic merge failed; fix conflicts and then commit the result.
