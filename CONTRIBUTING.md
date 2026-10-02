# Contributing

Changes come in as pull requests on
<https://github.com/leroyguillaume/cinelist>. The short version: install the
tools below, run `pre-commit install`, keep `pre-commit run --all-files` and
the test suites green.

## Development setup

Running the project needs what the [README](README.md#getting-started) lists.
Working on it needs a few more tools, which the pre-commit hooks call directly.

- `pre-commit` 4:

  ```sh
  pipx install pre-commit
  ```

- Helm and `helm-docs` 1.14, `hadolint` 2.15, Trivy, `actionlint` 1.7 and
  ShellCheck, installed from each project's release page:
  - Helm: <https://helm.sh/docs/intro/install/>
  - `helm-docs`: <https://github.com/norwoodj/helm-docs/releases>
  - `hadolint`: <https://github.com/hadolint/hadolint/releases>
  - Trivy: <https://trivy.dev/latest/getting-started/installation/>
  - `actionlint`: <https://github.com/rhysd/actionlint/releases>
  - ShellCheck: <https://github.com/koalaman/shellcheck#installing>
- `cargo-edit`, only to cut releases:

  ```sh
  cargo install cargo-edit
  ```

Then, from a fresh clone:

```sh
npm ci --prefix web
```

```sh
pre-commit install
```

```sh
docker compose up -d --wait
```

The setup is good when `pre-commit run --all-files` and both test suites
below pass.

## Running the tests

### API

The Rust tests need the `docker-compose` database; `.cargo/config.toml`
points them at it. Each integration test gets its own freshly migrated
database through `#[sqlx::test]`.

```sh
cargo test --workspace
```

- Unit tests of the game rules live next to them, in
  [api/src/domain.rs](api/src/domain.rs):

  ```sh
  cargo test -p cinelist-api --lib
  ```

- Integration tests drive the whole HTTP router against Postgres, in
  [api/tests/api.rs](api/tests/api.rs). One of them:

  ```sh
  cargo test -p cinelist-api --test api a_selection_runs_to_a_watched_film
  ```

### Web

The front's tests run with Vitest and Testing Library, in jsdom, with `fetch`
stubbed; they need nothing running.

```sh
npm --prefix web test
```

One file while iterating:

```sh
npm --prefix web test -- src/pages/SelectionPage.test.tsx
```

A new behaviour comes with a test, and a fix with a test that failed before it.

## Pre-commit hooks

Hooks are defined in [.pre-commit-config.yaml](.pre-commit-config.yaml) and
must pass before a change is pushed. `--no-verify` and `SKIP=` are not the
fix: a red hook is a real finding. If a rule is genuinely wrong for this
repository, change its configuration in the same pull request and say why.

```sh
pre-commit run --all-files
```

```sh
pre-commit run <hook-id> --all-files
```

| Hook | Checks | Fix |
| --- | --- | --- |
| `trailing-whitespace`, `end-of-file-fixer` | whitespace | re-run, it fixes itself |
| `check-yaml`, `yamllint` | YAML syntax and style ([.yamllint.yaml](.yamllint.yaml)) | block style, no `---` |
| `check-added-large-files`, `check-merge-conflict`, `detect-private-key` | accidental commits | remove the file or the marker |
| `sqlfluff-lint` | SQL style ([.sqlfluff](.sqlfluff)) | `sqlfluff fix <file>` |
| `biome-check` | TypeScript format and lint ([biome.json](biome.json)) | `npm --prefix web exec biome -- check --write .` |
| `tsc` | strict type checking of the front | fix the types |
| `cargo-fmt`, `cargo-clippy` | Rust format, pedantic lints as errors | `cargo fmt --all`, then fix the lints |
| `hadolint`, `trivy-dockerfile` | Dockerfiles | fix the Dockerfile |
| `helm-lint`, `helm-docs`, `trivy-chart` | the chart, its generated README, its security context | re-run `helm-docs`; fix the template |
| `actionlint-system` | GitHub workflows | fix the workflow |
| `shellcheck` | shell scripts | fix the script |
| `no-co-authors` | commit messages carry no co-author or "generated with" line | reword the commit |

Trivy reads [trivy/data/](trivy/data) (registries trusted by KSV-0125) and
[.trivyignore.yaml](.trivyignore.yaml), where each check the chart does not
satisfy carries its reason.

## Continuous integration

| Workflow | Triggers on | What it does | Reproduce locally |
| --- | --- | --- | --- |
| [quality](.github/workflows/quality.yaml) | every pull request, `main` | pre-commit on all files; Rust tests against a Postgres service; web tests and build | `pre-commit run --all-files`, the test commands above, `npm --prefix web run build` |
| [build](.github/workflows/build.yaml) | pull requests and `main` touching `api/`, `web/` or the Cargo manifests; manual; called by `release` | builds both images for amd64 and arm64 on native runners, scans them with Trivy and reports to code scanning, pushes them only when asked, from a tag or `main` | `docker build -f api/Dockerfile .`, `docker build -f web/Dockerfile .` |
| [security](.github/workflows/security.yaml) | every pull request, `main`, nightly | Trivy scan of the repository and of the latest published images, reported to code scanning | `trivy fs .` |
| [release](.github/workflows/release.yaml) | tag `v*` | checks the tag matches the committed versions, publishes both images; for a stable tag, tags them `latest` and creates the GitHub release, with notes since the previous stable tag | — |
| [chart](.github/workflows/chart.yaml) | tag `chart-*`, manual | checks the tag matches `Chart.yaml` and, for a stable tag, the `helm install` command of the README; publishes the chart to GHCR | `helm package charts/cinelist` |

`quality` and `security` must be green to merge; `build` only runs when its
inputs change, so it is not a required check. Fixable high and critical
vulnerabilities fail `build` and `security`; the rest is reported without
blocking. The publishing jobs use only the workflow's own `GITHUB_TOKEN`,
with `packages: write` to push to GHCR.

Dependabot keeps the actions, crates, npm packages and images up to date
([.github/dependabot.yaml](.github/dependabot.yaml)); its pull requests are
reviewed and merged by hand.

## Submitting a change

- Commit subjects are imperative and lowercase, with an optional `area:`
  prefix; follow `git log --oneline`.
- A pull request keeps CI green and updates the documentation it affects in
  the same change.
- Label it `feature`, `fix`, `security`, `performance`, `deprecation`,
  `documentation`, `dependencies` or `chore`, plus `breaking` when it breaks
  something, so it lands in the right section of the release notes
  ([.github/release.yaml](.github/release.yaml)).

## Releasing

Releases are cut with [scripts/release.sh](scripts/release.sh), from an
up-to-date `main` with a clean tree. It writes the version, commits, tags, and
asks before pushing; the pushed tag is what publishes.

The application — both images — is tagged `vX.Y.Z`:

```sh
scripts/release.sh 1.2.0
```

The chart has its own version, tagged `chart-X.Y.Z`, and deploys the latest
application release unless told otherwise:

```sh
scripts/release.sh --chart 0.3.0
```

`--dry-run` shows what would happen without changing anything.
