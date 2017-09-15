#!/usr/bin/env bash
#
# Update each feed.
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

[ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"
export PATH="${PATH}:$(dirname "$0")"

set -o pipefail

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

feed_tmp() {
    # Create a new per-feed temporary file, warning on failure.
    local file="$1/${2}"

    mktemp "${file}"
    if [ "$?" -ne 0 ]; then
        printf "%s: failed to create temporary file '%s'\n" "$0" "${file}" \
            1>&2
        return 1
    fi
}

update_feed() {
    # Update the feed with the given directory.
    local feed="$1"

    printf '%s: updating feed %s\n' "$0" "$(basename "${feed}")"

    need_dir "${feed}/read/" || return 1
    need_dir "${feed}/unread/" || return 1
    need_exec "${feed}" "fetch" || return 1

    local new_feed="${tmpdir}/feed"
    local old_feed="${feed}/feed"
    local new_entries="${tmpdir}/new-entries"
    local old_entries="${tmpdir}/old-entries"

    "${feed}/fetch" > "${new_feed}" 2> "${tmpdir}/output"
    if [ "$?" -ne 0 ]; then
        cat "${tmpdir}/output" 1>&2
        mv -f "${tmpdir}/output" "${feed}/error.log"
        printf "%s: failed to fetch new feed for '%s'\n" "$0" "${feed}" 1>&2
        return 1
    fi
    rm -f "${feed}/error.log"

    atom-list < "${new_feed}" | LC_ALL="C" sort > "${new_entries}"
    if [ "$?" -ne 0 ]; then
        printf "%s: failed to list new entries\n" "$0" 1>&2
        return 1
    fi
    # Sanity check the new feed file.
    if [ "$(wc -l < "${new_entries}")" -eq 0 ]; then
        printf "%s: new feed file contains no entries\n" "$0" 1>&2
        return 1
    fi
    if [ -e "${old_feed}" ]; then
        atom-list < "${old_feed}" | LC_ALL="C" sort > "${old_entries}"
        if [ "$?" -ne 0 ]; then
            printf "%s: failed to list entries in feed '%s'\n" "$0" "${feed}" \
                1>&2
            return 1
        fi
    else
        : > "${old_entries}" # Make an empty file if this is a new feed.
    fi

    # Add any new entries to the unread folder.
    LC_ALL="C" comm -23 "${new_entries}" "${old_entries}" |
    while IFS="\n" read -r entry; do
        local entry_name
        entry_name="$(feed-unescape "${entry}")"
        if [ "$?" -ne 0 ]; then
            printf "%s: unescaping failed\n" "$0" 1>&2
            return 1
        fi

        # Extract the entry.
        mkdir "${feed}/unread/${entry}" && \
        atom-extract "${entry_name}" < "${new_feed}" \
            > "${feed}/unread/${entry}/entry"
        if [ "$?" -ne 0 ]; then
            printf "%s: extracting entry from %s failed\n" "$0" "${new_feed}" \
                1>&2
            return 1
        fi

        # Cache the entry.
        if [ -x "${feed}/cache" ]; then
            atom-exec "${feed}/unread/${entry}/entry" \
                "${feed}/cache" "${feed}/unread/${entry}" \
                > "${tmpdir}/output" 2>&1
            if [ "$?" -ne 0 ]; then
                cat "${tmpdir}/output" 1>&2
                printf "%s: caching entry '%s' failed\n" "$0" "${entry}" 1>&2
            fi
        fi
    done

    # Move the new feed over the old one.
    mv -f "${new_feed}" "${old_feed}"

    # Clean up old entries.
    LC_ALL="C" comm -23 \
        <(cd "${feed}/read/"; printf '%s\n' * | LC_ALL="C" sort) \
        "${new_entries}" |
    while IFS="\n" read -r entry; do
        rm -rf "${feed}/read/${entry}"
    done

    tput cuu1
    tput el
}

# Create our (global) temporary directory.
tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/feed-update.XXXX")"
if [ "$?" -ne 0 ]; then
    printf "%s: failed to create temporary directory\n" "$0" 1>&2
    exit 1
fi
trap "rm -rf '${tmpdir}'" EXIT


for feed in "${FEED_DIR}/"*; do
    [ -d "${feed}" ] && update_feed "${feed}"
done

exit 0
