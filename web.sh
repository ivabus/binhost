#!/bin/sh
# SPDX-License-Identifier: MIT
set -e -o pipefail

NAME="{{NAME}}"

if ! uname > /dev/null; then
  echo "No \`uname\` command was found, cannot continue" 1>&2
  exit 1
fi

if ! expr "{{PLATFORM_LIST}}" : "\(.*$(uname)-$(uname -m).*\)" > /dev/null; then
  echo Platform $(uname)-$(uname -m) is not supported 1>&2
  exit 1
fi

DOWNLOAD_COMMAND="curl"
OUTPUT_ARG="-o"
DIR="/tmp/binhost-$NAME-$(date +%s)"
FILE="$DIR/$NAME"

if ! which curl > /dev/null; then
  if ! which wget > /dev/null; then
    echo "No curl or wget found, install one and rerun the script" 1>&2
    exit 1
  fi
  export DOWNLOAD_COMMAND="wget"
  export OUTPUT_ARG="-O"
fi

mkdir $DIR

echo ":: Downloading binary" 1>&2

# shellcheck disable=SC1083
$DOWNLOAD_COMMAND {{EXTERNAL_ADDRESS}}/bin/$NAME/$(uname)/$(uname -m) $OUTPUT_ARG "$FILE"

chmod +x "$FILE"

cd $DIR
{{HASH_CALCULATION}}
$FILE < /dev/tty

rm "$FILE"
