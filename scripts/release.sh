#!/usr/bin/env bash
# Cuts a release: bumps the version in the repository, commits, tags, and asks
# before pushing. Pushing the tag is what makes CI publish.
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/release.sh [--dry-run] <version>
       scripts/release.sh [--dry-run] --chart [--app-version <version>] <version>

Releases the application (tag vX.Y.Z: API and web images), or with --chart the
Helm chart (tag chart-X.Y.Z), which versions on its own.

Options:
  --chart                  release the Helm chart instead of the application
  --app-version <version>  application version the chart deploys
                           (default: the latest v* tag)
  --dry-run                show what would change, change nothing
  -h, --help               show this help

Examples:
  scripts/release.sh 1.2.0
  scripts/release.sh --chart 0.3.0
  scripts/release.sh --chart --app-version 1.1.0 0.3.1
  scripts/release.sh --dry-run 1.2.1
USAGE
}

die() {
  echo "error: $*" >&2
  exit 1
}

chart=false
dry_run=false
app_version=""
version=""
while [ $# -gt 0 ]; do
  case "$1" in
    --chart) chart=true ;;
    --dry-run) dry_run=true ;;
    --app-version)
      [ $# -ge 2 ] || die "--app-version needs a value"
      app_version="$2"
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    -*) die "unknown option $1 (see --help)" ;;
    *)
      [ -z "$version" ] || die "only one version can be given"
      version="$1"
      ;;
  esac
  shift
done

semver='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-[0-9A-Za-z.-]+)?$'
[ -n "$version" ] || die "a version is required (see --help)"
[[ "$version" =~ $semver ]] || die "${version} is not a semantic version (X.Y.Z)"
[ -z "$app_version" ] || $chart || die "--app-version only applies to --chart"

cd "$(git rev-parse --show-toplevel)"
git rev-parse --quiet --verify HEAD > /dev/null || die "the repository has no commit yet"

branch="$(git rev-parse --abbrev-ref HEAD)"
default_branch="$(git symbolic-ref --quiet --short refs/remotes/origin/HEAD || true)"
default_branch="${default_branch#origin/}"
default_branch="${default_branch:-main}"
[ "$branch" = "$default_branch" ] || die "releases are cut from ${default_branch}, not ${branch}"
[ -z "$(git status --porcelain)" ] || die "the working tree is not clean; commit or stash first"
git fetch --quiet --tags origin "$branch"
behind="$(git rev-list --count "HEAD..origin/${branch}")"
[ "$behind" -eq 0 ] || die "${branch} is ${behind} commit(s) behind origin; pull first"

chart_file=charts/cinelist/Chart.yaml
if $chart; then
  tag="chart-${version}"
  if [ -z "$app_version" ]; then
    latest="$(git tag --list 'v*' --sort=-version:refname | head -n 1)"
    [ -n "$latest" ] || die "no application release (v*) exists yet; pass --app-version"
    app_version="${latest#v}"
  fi
  [[ "$app_version" =~ $semver ]] || die "${app_version} is not a semantic version"
  git rev-parse --quiet --verify "refs/tags/v${app_version}" > /dev/null \
    || die "v${app_version} is not a released application version"
  current="$(sed -n 's/^version: *//p' "$chart_file")"
  subject="chart: release ${version}"
else
  tag="v${version}"
  current="$(sed -n 's/^version = "\(.*\)"$/\1/p' api/Cargo.toml)"
  subject="release ${version}"
fi
! git rev-parse --quiet --verify "refs/tags/${tag}" > /dev/null || die "tag ${tag} already exists"

if $dry_run; then
  echo "current version: ${current}"
  echo "target version:  ${version}"
  $chart && echo "app version:     ${app_version}"
  echo "commit:          ${subject}"
  echo "tag:             ${tag}"
  exit 0
fi

if $chart; then
  sed -i.bak -e "s/^version: .*/version: ${version}/" -e "s/^appVersion: .*/appVersion: \"${app_version}\"/" "$chart_file"
  rm "${chart_file}.bak"
  helm-docs --chart-search-root charts
  git add charts/cinelist
  if [ "${version#*-}" = "$version" ]; then
    sed -i.bak -e "s|\(/charts/cinelist --version \)[^ ]*|\1${version}|" README.md
    rm README.md.bak
    git add README.md
  fi
elif [ "$current" != "$version" ]; then
  command -v cargo-set-version > /dev/null || die "cargo set-version is missing: cargo install cargo-edit"
  cargo set-version --workspace "$version"
  npm --prefix web version "$version" --no-git-tag-version > /dev/null
  git add Cargo.lock api/Cargo.toml web/package.json web/package-lock.json
fi
# The first release ships the version the repository was created with: there is
# nothing to commit, and the tag goes on HEAD.
if git diff --cached --quiet; then
  git tag --annotate "$tag" --message "$subject"
  echo "${version} is already committed; tagged HEAD ${tag}"
else
  git commit --quiet --message "$subject"
  git tag --annotate "$tag" --message "$subject"
  echo "committed \"${subject}\" and tagged ${tag}"
fi

push_command="git push --atomic origin ${branch} ${tag}"
if [ -t 0 ]; then
  printf 'push %s and %s to origin? this publishes the release. [y/N] ' "$branch" "$tag"
  read -r reply
else
  reply=""
fi
case "$reply" in
  y | Y | yes)
    $push_command
    ;;
  *)
    echo "not pushed. publish later with: ${push_command}"
    ;;
esac
