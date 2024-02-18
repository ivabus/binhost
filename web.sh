#!/bin/sh
# SPDX-License-Identifier: MIT
set -e

print() {
  echo "$@" >&2
}

fail() {
  print "$@"
  exit 1
}

requireCommands() {
  for cmd in $*; do
    if ! command -v $cmd &> /dev/null; then
      fail "Cannot find required command: $cmd"
    fi
  done
}

requireCommands uname cut dd chmod rm realpath expr

# Finding alternative, but supported sha256sums
SHA256SUM=""
if command -v sha256sum &> /dev/null; then
  export SHA256SUM="sha256sum"
else
  if command -v shasum &> /dev/null; then
    SHASUMVER=$(shasum -v | cut -c 1)
    if [ $SHASUMVER -ge 6 ]; then
      export SHA256SUM="shasum -a 256"
    fi
  else
    fail "Could not find sha256sum executable"
  fi
fi

if [ "$(realpath $SHA256SUM)" = "/bin/busybox" ]; then
  fail "Busybox sha256sum detected, will not work. Refusing to continue"
fi

# Script could be used for any binary
NAME=${BIN:="{{NAME}}"}
EXTERNAL_ADDRESS=${EXTERNAL_ADDRESS:="{{EXTERNAL_ADDRESS}}"}
DOWNLOAD_COMMAND="curl"
OUTPUT_ARG="-o"
DIR="/tmp/binhost-$NAME-$(date +%s)"
FILE="$DIR/$NAME"
PLATFORM="$(uname)"
ARCH="$(uname -m)"

if ! command -v curl &> /dev/null; then
  if ! command -v wget &> /dev/null; then
    fail "No curl or wget found, install one and rerun the script"
  fi
  export DOWNLOAD_COMMAND="wget"
  export OUTPUT_ARG="-O"
fi

PLATFORM_LIST="{{PLATFORM_LIST}}"
# Making script truly portable
if [ ! "{{NAME}}" = $NAME ]; then
  print ":: Fetching platforms"
  export PLATFORM_LIST=$($DOWNLOAD_COMMAND $EXTERNAL_ADDRESS/$NAME/platforms $OUTPUT_ARG /dev/stdout)
fi

if ! expr "$PLATFORM_LIST" : "\(.*$(uname)-$(uname -m).*\)" > /dev/null; then
  fail "Platform \"$(uname)-$(uname -m)\" is not supported"
fi

mkdir $DIR
cd $DIR

print ":: Downloading manifest"
$DOWNLOAD_COMMAND $EXTERNAL_ADDRESS/runner/manifest $OUTPUT_ARG manifest

MANIFEST_HASHSUM=$($SHA256SUM manifest)

if [ ! -z $KEY ]; then
  if [ ! $KEY = "$(echo $MANIFEST_HASHSUM | cut -c 1-${#KEY})" ]; then
    fail "Invalid manifest hashsum"
  fi
else
  print "Manifest KEY missing, skipping manifest check"
fi

print ":: Downloading signature"
$DOWNLOAD_COMMAND $EXTERNAL_ADDRESS/bin/$NAME/$PLATFORM/$ARCH/sign $OUTPUT_ARG signature

dd if=manifest of=public_key count=32 bs=1 2> /dev/null
dd if=manifest of=hashes skip=32 bs=1 2> /dev/null

print ":: Downloading runner"

$DOWNLOAD_COMMAND $EXTERNAL_ADDRESS/runner/runner-$PLATFORM-$ARCH $OUTPUT_ARG "runner-$PLATFORM-$ARCH"

if ! $SHA256SUM -c hashes --ignore-missing >&2 ; then
  fail "Incorrect hashsum of runner"
fi

chmod +x "runner-$PLATFORM-$ARCH"

print ":: Downloading binary"

$DOWNLOAD_COMMAND $EXTERNAL_ADDRESS/bin/$NAME/$PLATFORM/$ARCH $OUTPUT_ARG "$FILE"

if ! ./runner-$PLATFORM-$ARCH "$FILE" >&2; then
  exit 2
fi

chmod +x "$FILE"

$FILE $ARGS < /dev/tty

cd

rm -rf "$DIR"
