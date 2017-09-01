#!/usr/bin/env sh
#
# Open each unread feed.
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

[ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"
export PATH="${PATH}:$(dirname "$0")"

if [ ! -d "${FEED_DIR}" ]; then
    printf "%s: feed dir '%s' does not exist\n" "$0" "${FEED_DIR}" 1>&2
    exit 1
fi

if [ $# -ne 0 ]; then
    printf "usage: %s\n" "$0" 1>&2
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

need_exec() {
    # Check that the given executable exists, warning on failure.
    local feed="$1"
    local exec="$2"

    if [ ! -x "${feed}/${exec}" ]; then
        printf "%s: feed '%s' has no %s executable\n" "$0" "${feed}" "${exec}" \
            1>&2
        return 1
    fi
}

read_feed() {
    # Read all unread entries for the feed with the given directory.
    local feed="$1"

    need_dir "${feed}/read/" || return
    need_dir "${feed}/unread/" || return
    need_exec "${feed}" "open" || return

    for unread in "${feed}/unread/"*; do
        if [ -e "${unread}/entry" ]; then
            atom-exec "${unread}/entry" \
                "${feed}/open" "${unread}"
            if [ "$?" -ne 0 ]; then
                printf "%s: failed to open %s\n" "$0" "${unread}" 1>&2
            else
                rm -rf "${feed}/read/$(basename "${unread}")"
                mv -f "${unread}" "${feed}/read/"
            fi
        fi
    done
}

for feed in "${FEED_DIR}/"*; do
    [ -d "${feed}" ] && read_feed "${feed}"
done
