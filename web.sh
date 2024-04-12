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
  for cmd in "$@"; do
    if ! command -v "$cmd" > /dev/null 2>&1; then
      fail "Cannot find required command: $cmd"
    fi
  done
}

requireCommands uname cut dd chmod rm realpath expr

# Finding alternative, but supported sha256sums
SHA256SUM=""
SHASUMFLAGS=""
PLATFORM="$(uname)"
ARCH="$(uname -m)"
if command -v sha256sum > /dev/null 2>&1; then
   SHA256SUM="sha256sum"
   SHASUMFLAGS="-c hashes --ignore-missing"
else
  if command -v sha256 > /dev/null 2>&1; then
      SHA256SUM="sha256"
      SHASUMFLAGS="-C hashes runner-$PLATFORM-$ARCH"
  fi
  if command -v shasum > /dev/null 2>&1; then
    SHASUMVER=$(shasum -v | cut -c 1)
    if [ "$SHASUMVER" -ge 6 ]; then
      SHA256SUM="shasum -a 256"
      SHASUMFLAGS="-c hashes --ignore-missing"
    fi
  fi
fi

if [ SHA256SUM = "" ]; then
    fail "Could not find suitable sha256sum executable"
fi

if [ "$(realpath "$SHA256SUM" 2> /dev/null)" = "/bin/busybox" ]; then
  fail "Busybox sha256sum detected, will not work. Refusing to continue"
fi

# Script could be used for any binary
NAME=${BIN:="{{NAME}}"}
EXTERNAL_ADDRESS=${EXTERNAL_ADDRESS:="{{EXTERNAL_ADDRESS}}"}
DOWNLOAD_COMMAND="curl"
OUTPUT_ARG="-o"
DIR="/tmp/binhost-$NAME-$(date +%s)"
FILE="$DIR/$NAME"

if ! command -v curl > /dev/null 2>&1; then
  if ! command -v wget > /dev/null 2>&1; then
    fail "No curl or wget found, install one and rerun the script"
  fi
  DOWNLOAD_COMMAND="wget"
  OUTPUT_ARG="-O"
fi

PLATFORM_LIST="{{PLATFORM_LIST}}"
# Making script truly portable
if [ ! "{{NAME}}" = $NAME ]; then
  print ":: Fetching platforms"
  PLATFORM_LIST=$($DOWNLOAD_COMMAND $EXTERNAL_ADDRESS/$NAME/platforms $OUTPUT_ARG /dev/stdout)
fi

if ! expr "$PLATFORM_LIST" : "\(.*$(uname)-$(uname -m).*\)" > /dev/null; then
  fail "Platform \"$(uname)-$(uname -m)\" is not supported"
fi

mkdir "$DIR"
cd "$DIR"

print ":: Downloading manifest"
$DOWNLOAD_COMMAND $EXTERNAL_ADDRESS/runner/manifest $OUTPUT_ARG manifest

MANIFEST_HASHSUM=$(cat manifest | $SHA256SUM)

if [ -n "$KEY" ]; then
  if [ ! "$KEY" = "$(echo "$MANIFEST_HASHSUM" | cut -c 1-${#KEY})" ]; then
    fail "Invalid manifest hashsum"
  fi
else
  print "Manifest KEY missing, skipping manifest check"
fi

print ":: Downloading signature"
$DOWNLOAD_COMMAND "$EXTERNAL_ADDRESS/bin/$NAME/$PLATFORM/$ARCH/sign" $OUTPUT_ARG signature

dd if=manifest of=public_key count=32 bs=1 2> /dev/null
dd if=manifest of=hashes skip=32 bs=1 2> /dev/null

print ":: Downloading runner"

$DOWNLOAD_COMMAND "$EXTERNAL_ADDRESS/runner/runner-$PLATFORM-$ARCH" $OUTPUT_ARG "runner-$PLATFORM-$ARCH"

if ! $SHA256SUM $SHASUMFLAGS >&2 ; then
  fail "Incorrect hashsum of runner"
fi

chmod +x "runner-$PLATFORM-$ARCH"

print ":: Downloading binary"

$DOWNLOAD_COMMAND "$EXTERNAL_ADDRESS/bin/$NAME/$PLATFORM/$ARCH" $OUTPUT_ARG "$FILE"

if ! "./runner-$PLATFORM-$ARCH" "$FILE" >&2; then
  exit 2
fi

chmod +x "$FILE"

$FILE $ARGS < /dev/tty

cd

rm -rf "$DIR"
