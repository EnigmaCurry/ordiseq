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

# Build and bundle a specific plugin
build-plugin NAME *args:
    cargo run --package xtask --release -- bundle {{NAME}} --release

# Build and bundle all plugins
build-plugins *args:
    just build-plugin melody_player_plug
    just build-plugin midi_inverter_plug
    just build-plugin ordiseq_euclid_plug
    just build-plugin ordiseq_topograph_plug
    @echo "All plugins built and bundled!"
    @ls -lh target/bundled/

# Package all plugins for distribution (rebuilds plugins first)
package-plugins: build-plugins
    just _package-plugins-only

# Package existing plugin builds without rebuilding
package-plugins-quick:
    just _package-plugins-only

# Internal: Package plugins from target/bundled
_package-plugins-only:
    #!/usr/bin/env bash
    set -euo pipefail

    # Get version from plugin Cargo.toml
    VERSION=$(grep '^version' plugins/melody_player_plug/Cargo.toml | head -1 | cut -d'"' -f2)
    DIST_DIR="target/dist"
    PACKAGE_NAME="ordiseq-plugins-v${VERSION}"
    PACKAGE_DIR="${DIST_DIR}/${PACKAGE_NAME}"

    echo "Creating distribution package: ${PACKAGE_NAME}"

    # Clean and create distribution directory
    rm -rf "${DIST_DIR}"
    mkdir -p "${PACKAGE_DIR}"/{VST3,CLAP}

    # Copy plugins
    echo "Copying plugins..."
    cp target/bundled/melody_player_plug.clap "${PACKAGE_DIR}/CLAP/"
    cp target/bundled/midi_inverter_plug.clap "${PACKAGE_DIR}/CLAP/"
    cp target/bundled/ordiseq_euclid_plug.clap "${PACKAGE_DIR}/CLAP/"
    cp target/bundled/ordiseq_topograph_plug.clap "${PACKAGE_DIR}/CLAP/"
    cp -r target/bundled/melody_player_plug.vst3 "${PACKAGE_DIR}/VST3/"
    cp -r target/bundled/midi_inverter_plug.vst3 "${PACKAGE_DIR}/VST3/"
    cp -r target/bundled/ordiseq_euclid_plug.vst3 "${PACKAGE_DIR}/VST3/"
    cp -r target/bundled/ordiseq_topograph_plug.vst3 "${PACKAGE_DIR}/VST3/"

    # Copy documentation
    echo "Copying documentation..."
    cp PLUGINS.md "${PACKAGE_DIR}/README.md"
    cp LICENSE.txt "${PACKAGE_DIR}/" 2>/dev/null || echo "No LICENSE file found"

    # Create installation instructions
    {
        echo "ORDISEQ PLUGIN SUITE INSTALLATION INSTRUCTIONS"
        echo ""
        echo "This package contains four plugins:"
        echo "- Melody Player: Plays Jingle Bells and Row Row Your Boat"
        echo "- MIDI Inverter: Inverts MIDI notes around a center point"
        echo "- Ordiseq Euclid: Euclidean rhythm generator with Length, Hits, and Rotate controls"
        echo "- Ordiseq Topograph: Topographic drum sequencer based on Mutable Instruments Grids"
        echo ""
        echo "INSTALLATION:"
        echo ""
        echo "Windows:"
        echo "  VST3: Copy VST3 folder contents to C:\\Program Files\\Common Files\\VST3\\"
        echo "  CLAP: Copy CLAP folder contents to C:\\Program Files\\Common Files\\CLAP\\"
        echo ""
        echo "macOS:"
        echo "  VST3: Copy VST3 folder contents to ~/Library/Audio/Plug-Ins/VST3/"
        echo "  CLAP: Copy CLAP folder contents to ~/Library/Audio/Plug-Ins/CLAP/"
        echo ""
        echo "Linux:"
        echo "  VST3: Copy VST3 folder contents to ~/.vst3/"
        echo "  CLAP: Copy CLAP folder contents to ~/.clap/"
        echo ""
        echo "After installation, rescan plugins in your DAW."
        echo ""
        echo "For detailed usage instructions, see README.md"
        echo ""
        echo "Website: https://github.com/EnigmaCurry/ordiseq"
    } > "${PACKAGE_DIR}/INSTALL.txt"

    # Create archive
    echo "Creating archive..."
    cd "${DIST_DIR}"

    # Detect platform and create appropriate archive
    if command -v zip >/dev/null 2>&1; then
        zip -r "${PACKAGE_NAME}.zip" "${PACKAGE_NAME}"
        echo "Created: ${DIST_DIR}/${PACKAGE_NAME}.zip"
    fi

    if command -v tar >/dev/null 2>&1; then
        tar -czf "${PACKAGE_NAME}.tar.gz" "${PACKAGE_NAME}"
        echo "Created: ${DIST_DIR}/${PACKAGE_NAME}.tar.gz"
    fi

    cd - >/dev/null

    echo ""
    echo "Distribution package ready in: ${DIST_DIR}/"
    ls -lh "${DIST_DIR}"/*.{zip,tar.gz} 2>/dev/null || ls -lh "${DIST_DIR}/"
    echo ""
    echo "Package contents:"
    ls -R "${PACKAGE_DIR}"

# Run ordiseq_app in development mode
run-app:
    @if [ ! -d ordiseq_app/node_modules ]; then cd ordiseq_app && npm install; fi
    @if [ ! -d ordiseq_app/frontend/node_modules ]; then cd ordiseq_app/frontend && npm install; fi
    cd ordiseq_app && npm run tauri dev

# Build ordiseq_app frontend and Rust backend
build-app:
    cd ordiseq_app/frontend && npm install && npm run build
    cargo build -p ordiseq_app

# Bundle ordiseq_app for distribution
bundle-app:
    cd ordiseq_app && npm run tauri build

# Bundle ordiseq_app for Windows (cross-compile from Linux)
bundle-app-windows:
    @if ! command -v cargo-xwin >/dev/null; then cargo install cargo-xwin; fi
    @if ! rustup target list --installed | grep -q x86_64-pc-windows-msvc; then rustup target add x86_64-pc-windows-msvc; fi
    cd ordiseq_app/frontend && npm run build
    cd ordiseq_app && cargo xwin build --release --target x86_64-pc-windows-msvc
    @echo "Windows binary: target/x86_64-pc-windows-msvc/release/ordiseq_app.exe"

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
