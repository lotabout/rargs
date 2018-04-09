#!/bin/bash

case $TRAVIS_OS_NAME in
    linux)
        pyenv global 3.6.3
        python3 -V
        python3.6 -V
        ;;
    osx)
        # install python 3.6
        brew upgrade python
        python3 -V
        python3.6 -V
        ;;
esac
