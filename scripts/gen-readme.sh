#!/bin/sh
set -e

cd "$(dirname $0)"/..
cat readme-parts/{header,main,license}.md > README.md
