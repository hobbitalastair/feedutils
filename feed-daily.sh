#!/usr/bin/env sh
#
# Read unread daily feeds.
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

[ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"
export PATH="${PATH}:$(dirname "$0")"

if [ ! -d "${FEED_DIR}" ]; then
    printf "%s: feed dir '%s' does not exist\n" "$0" "${FEED_DIR}" 1>&2
    exit 1
fi

if [ "$#" -ne 0 ]; then
    printf 'usage: %s\n' "$0" 1>&2
    exit 1
fi

for feed in "${FEED_DIR}/"*; do
    [ -d "${feed}" ] && [ -f "${feed}/daily" ] && feed-read "${feed##*/}"
done
