set export

current_dir := `pwd`
RUST_LOG := "debug"
RUST_BACKTRACE := "1"

# print help for Just targets
help:
    @just -l

# Install dependencies
deps:
    @if ! command -v cargo-workspace >/dev/null; then \
        cargo install --locked cargo-workspace; \
    fi
    @if ! command -v cargo-watch >/dev/null; then \
        cargo install --locked cargo-watch; \
    fi
    @if ! command -v cargo-nextest >/dev/null; then \
        cargo install --locked cargo-nextest; \
    fi
    @if ! command -v git-cliff >/dev/null; then \
        cargo install --locked git-cliff; \
    fi
    @if ! command -v cargo-llvm-cov >/dev/null; then \
        cargo install --locked cargo-llvm-cov; \
    fi
    @if ! command -v live-server >/dev/null; then \
        cargo install --locked live-server; \
    fi

# Install binary dependencies (gh-actions)
bin-deps:
    cargo binstall --no-confirm cargo-workspace
    cargo binstall --no-confirm cargo-nextest
    cargo binstall --no-confirm cargo-llvm-cov

# Build and run binary + args
run *args:
    cargo run --manifest-path "${current_dir}/Cargo.toml" {{args}}

# Build + args
build *args:
    RUSTFLAGS="-D warnings" cargo build {{args}}

build-plugin *args:
    cd ordiseq-plug && cargo xtask bundle ordiseq-plug --release

# Run tests
test *args: 
    cargo nextest run {{args}}

# Run tests continuously on file change
test-watch *args: 
    cargo watch -s "clear && cargo nextest run {{args}}"

# Run tests with verbose logging
test-verbose *args:
    RUST_TEST_THREADS=1 cargo nextest run --nocapture {{args}}

# Run tests continuously with verbose logging
test-watch-verbose *args:
    RUST_TEST_THREADS=1 cargo watch -s "clear && cargo nextest run --nocapture -- {{args}}"

# Build coverage report
test-coverage *args: clean
    cargo llvm-cov --doctests test {{args}} && \
    cargo llvm-cov {{args}} report --html

# Continuously build coverage report and serve HTTP report
test-coverage-watch *args: 
    cargo watch -s "clear && just test-coverage {{args}} && cd target/llvm-cov/html && python -m http.server"

# Run Clippy to report and fix lints
clippy *args:
    RUSTFLAGS="-D warnings" cargo clippy {{args}} --color=always 2>&1 --tests | less -R

# Bump release version and create PR branch
bump-version: 
    @if [ -n "$(git status --porcelain)" ]; then echo "## Git status is not clean. Commit your changes before bumping version."; exit 1; fi
    @if [ "$(git symbolic-ref --short HEAD)" != "master" ]; then echo "## You may only bump the version from the master branch."; exit 1; fi
    source ./funcs.sh; \
    set -eo pipefail; \
    CURRENT_VERSION=$(grep -Po '^version = \K.*' Cargo.toml | sed -e 's/"//g' | head -1); \
    VERSION=$(git cliff --bumped-version | sed 's/^v//'); \
    echo; \
    (if git rev-parse v${VERSION} 2>/dev/null; then \
      echo "New version tag already exists: v${VERSION}" && \
      echo "If you need to re-do this release, delete the existing tag (git tag -d v${VERSION})" && \
      exit 1; \
     fi \
    ); \
    echo "## Current $(grep '^version =' Cargo.toml | head -1)"; \
    confirm yes "New version would be \"v${VERSION}\"" " -- Proceed?"; \
    git checkout -B release-v${VERSION}; \
    cargo set-version ${VERSION}; \
    sed -i "s/^VERSION=v.*$/VERSION=v${VERSION}/" README.md; \
    cargo update; \
    git add Cargo.toml Cargo.lock README.md; \
    git commit -m "release: v${VERSION}"; \
    echo "Bumped version: v${VERSION}"; \
    echo "Created new branch: release-v${VERSION}"; \
    echo "You should push this branch and create a PR for it."

# Tag and release a new version from master branch
release: 
    @if [ -n "$(git status --porcelain)" ]; then echo "## Git status is not clean. Commit your changes before bumping version."; exit 1; fi
    @if [ "$(git symbolic-ref --short HEAD)" != "master" ]; then echo "## You may only release the master branch."; exit 1; fi
    git remote update;
    @if [[ "$(git status -uno)" != *"Your branch is up to date"* ]]; then echo "## Git branch is not in sync with git remote ${GIT_REMOTE}."; exit 1; fi;
    @set -eo pipefail; \
    source ./funcs.sh; \
    CURRENT_VERSION=$(grep -Po '^version = \K.*' Cargo.toml | sed -e 's/"//g' | head -1); \
    if git rev-parse "v${CURRENT_VERSION}" >/dev/null 2>&1; then echo "Tag already exists: v${CURRENT_VERSION}"; exit 1; fi; \
    if (git ls-remote --tags "${GIT_REMOTE}" | grep -q "refs/tags/v${CURRENT_VERSION}" >/dev/null 2>&1); then echo "Tag already exists on remote ${GIT_REMOTE}: v${CURRENT_VERSION}"; exit 1; fi; \
    cargo audit | less; \
    confirm yes "New tag will be \"v${CURRENT_VERSION}\"" " -- Proceed?"; \
    git tag "v${CURRENT_VERSION}"; \
    git push "${GIT_REMOTE}" tag "v${CURRENT_VERSION}";

# Clean all artifacts
clean *args: clean-profile
    cargo clean {{args}}
    rm -f *.mid

# Clean profile artifacts only
clean-profile:
    rm -rf *.profraw *.profdata

# Build and serve documentation site
doc: 
    RUST_LOG=warn live-server target/doc --open=ordiseq & \
    cargo watch -s 'cargo doc'

# example-circle-of-fifths:
#     cargo run --example circle_of_fifths

# example-circle-of-fifths-rhythm:
#     cargo run --example circle_of_fifths_rhythm

# example-scale:
#     cargo run --example scale

# example-scale-omnibus:
#     cargo run --example scale_omnibus

# example-chord-progression:
#     cargo run --example chord_progression
