#!/usr/bin/env sh
#
# List unread entries for the given feed.
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

if [ "$#" -eq 1 ] && [ "$1" == '-s' ]; then
    "$0" | sed s:/.*:: | sort | uniq -c | sort -n
    exit "$?"
elif [ "$#" -ne 0 ]; then
    printf 'usage: %s [-s]\n' "$0" 1>&2
    exit 1
fi

cd "${FEED_DIR}"
for entry in */entry/*; do
    if [ ! -f "${entry}/read" ]; then
        printf '%s\n' "${entry}"
    fi
done
