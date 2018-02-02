#!/usr/bin/env sh
#
# Read unread feeds.
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

need_exec() {
    # Check that the given executable exists, warning on failure.
    local feed="$(dirname "$1")"
    local exec="$(basename "$1")"

    if [ ! -x "${feed}/${exec}" ]; then
        printf "%s: feed %s has no %s executable\n" "$0" "${feed}" "${exec}" \
            1>&2
        return 1
    fi
}

read_entry() {
    # Open the given entry.
    local feed="$1"
    local entry="$2"

    need_exec "${feed}/open" || return 1

    atom-exec "${entry}/entry" "${feed}/open" "${entry}"
    if [ "$?" -ne 0 ]; then
        printf "%s: failed to open %s\n" "$0" "${entry}" 1>&2
    else
        touch "${entry}/read"
    fi
}

read_feed() {
    # Read all unread entries for the feed with the given directory.
    local feed="$1"

    need_dir "${feed}/entry/" || return 1

    if [ -f "${feed}/error.log" ]; then
        printf "%s: %s has errors:\n" "$0" "$(basename "${feed}")" 1>&2
        cat < "${feed}/error.log" 1>&2
    fi

    # Preserve stdin and stdout.
    exec 3>&0
    exec 4>&1

    for entry in "${feed}/entry/"*; do
        if [ -f "${entry}/entry" ] && [ ! -f "${entry}/read" ]; then
            if [ -f "${entry}/timestamp" ]; then
                # We sort on timestamp, id.
                # Neither parts of the key should contain newlines or '/', so
                # use those as separators.
                printf '%s/%s\n' "$(cat "${entry}/timestamp" 2> /dev/null)" \
                    "${entry}"
            else
                # Fallback if we don't have a timestamp.
                read_entry "${feed}" "${entry}" <&3 >&4
            fi
        fi
    done | sort | \
    while IFS="$(printf '/\n')" read -r timestamp entry; do
        # Open the ordered entries.
        read_entry "${feed}" "${entry}" <&3 >&4
    done
}

if [ "$#" -ne 0 ]; then
    cd "${FEED_DIR}"
    for arg in "$@"; do
        if [ -f "${arg}/entry" ]; then
            # This is a path to a feed entry to open.
            read_entry "$(realpath "${arg}/../../")" "${arg}"
        elif [ -x "${arg}/fetch" ]; then
            # This is a path to a particular feed; open all unread.
            read_feed "${arg}"
        fi
    done
else
    # No feeds given; open all unread.
    for feed in "${FEED_DIR}/"*; do
        [ -d "${feed}" ] && read_feed "${feed}"
    done
fi
