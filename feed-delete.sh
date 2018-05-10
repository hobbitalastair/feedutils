#!/usr/bin/env sh
#
# Delete the given feed.
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

set -e

[ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"
export PATH="${PATH}:$(dirname "$0")"

if [ ! -d "${FEED_DIR}" ]; then
    printf "%s: feed dir '%s' does not exist\n" "$0" "${FEED_DIR}" 1>&2
    exit 1
fi

if [ "$#" -ne 1 ]; then
    printf 'usage: %s <name>\n' "$0" 1>&2
    exit 1
fi
name="$1"

cd "${FEED_DIR}"
if [ ! -d "${name}" ]; then
    printf "%s: no such feed: '%s'\n" "$0" "${name}" 1>&2
    exit 1
fi

rm -rf "${name}"
