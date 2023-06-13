export SHELL_COMPLETIONS_DIR := "./pkg/assets/completions"
export MANPAGES_DIR := "./pkg/assets/man"


rustc-version := "1.69.0-x86_64-unknown-linux-gnu"

cargo +args='':
    cargo {{args}}

pre-release:
    just cargo check \
        && just cargo test \
        && just cargo clippy \
        && just cargo fmt \
        && echo "Pre-Release checks passed!"

verify-clean-git:
    test "$(echo `git status --porcelain` | wc -c)" -eq "1"

get-crate-version:
    @cat Cargo.toml | rg '^version =' | sed -e 's/^version\s*=\s*//' | tr -d '"'

verify-release-tag-does-not-exist:
    VERSION=$(just get-crate-version) \
        && test -z "$(git tag | rg \"v${VERSION}\")" # Error: tag appears to exist already


clean:
  rm -rf ./target

build:
  cargo build

build-release:
  cargo build --release

install:
  cargo install --path .

package:
  just build-release
  rm -rf ./release
  mkdir ./release
  mv ./target/release ./release
  cp -r $MANPAGES_DIR ./release
  cp -r $SHELL_COMPLETIONS_DIR ./release

build_release: pre-release
  cargo clean
  cargo build --release --target x86_64-unknown-linux-gnu
  mkdir ./target/release/full-release
  cp -r pkg/assets/man ./target/release/full-release
  cp -r pkg/assets/completions ./target/release/full-release
  export VERSION=$(just get-crate-version)
  export RELEASE_PACKAGE="./target/release/powernotd-${VERSION}-x86_64.tar.gz"
  tar -czvf $RELEASE_PACKAGE -C target/release/full-release ./target/release/full-release/ 
  md5sum $RELEASE_PACKAGE > ./target/release/powernotd-${VERSION}-checksum.md5



# publish crate version to private registry
publish_crate +args='': verify-clean-git verify-release-tag-does-not-exist pre-release
    git push
    sleep 0.25
    cargo +{{rustc-version}} publish \
        --no-default-features {{args}}
    echo "adding git tag, now that EVERYTHING worked..."
    git tag "v$(just get-crate-version)"
    git push --tags
    rm -rf target/package
