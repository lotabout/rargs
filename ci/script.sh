# This script takes care of testing
set -ex

main() {
    cargo build --release --verbose

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cargo test --verbose

    python3.6 test/test.py
}

if [ -z $TRAVIS_TAG ]; then
    main
fi
