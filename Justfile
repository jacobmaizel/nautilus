default:
  just clippy
  just test
  gitleaks detect

test:
    ./scripts/tests.sh

citest:
  cargo nextest run

clippy:
  cargo clippy --workspace --all-targets --no-deps -- -D warnings

flame:
  CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph --root
