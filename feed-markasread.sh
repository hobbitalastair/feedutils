#!/usr/bin/env sh
#
# Mark all feeds as read.
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

[ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"
export PATH="${PATH}:$(dirname "$0")"

if [ ! -d "${FEED_DIR}" ]; then
    printf "%s: feed dir '%s' does not exist\n" "$0" "${FEED_DIR}" 1>&2
    exit 1
fi

need_dir() {
    # Create a directory if it doesn't already exist, warning on failure.
    local new_dir="$1"

    [ -d "${new_dir}" ] && return

    mkdir "${new_dir}"
    if [ "$?" -ne 0 ]; then
        printf "%s: failed to create directory '%s'\n" "$0" "${new_dir}" 1>&2
        return 1
    fi
}

mark_all_as_read() {
    # Mark all entries in the feed as read
    local feed="$1"

    need_dir "${feed}/entry/" || return 1

    for entry in "${arg}/entry/"*; do
        if [ -f "${entry}/entry" ]; then
            touch "${entry}/read"
        fi
    done
}

if [ "$#" -ne 0 ]; then
    cd "${FEED_DIR}"
    for arg in "$@"; do
        if [ -x "${arg}/fetch" ]; then
            mark_all_as_read "${arg}"
        fi
    done
else
    printf 'usage: %s <feed>\n' "$0" 1>&2
    exit 1
fi

