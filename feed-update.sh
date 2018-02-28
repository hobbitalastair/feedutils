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

log_update_feed() {
    # Wrapper for update_feed which also logs the output.
    local log="${tmpdir}/error.log"
    local feed="$1"
    rm -f "${log}"
    update_feed "${feed}" 2> >(tee -a "${log}")
    if [ "$?" -ne 0 ]; then
        mv "${tmpdir}/error.log" "${feed}/error.log"
    else
        rm -f "${feed}/error.log"
    fi
}

update_feed() {
    # Update the feed with the given directory.
    local feed="$1"

    printf '%s: updating feed %s\n' "$0" "$(basename "${feed}")"

    need_dir "${feed}/entry/" || return 1
    need_exec "${feed}" "fetch" || return 1

    local new_feed="${tmpdir}/feed"
    local new_entries="${tmpdir}/new-entries"
    local old_entries="${tmpdir}/old-entries"

    "${feed}/fetch" > "${new_feed}" 2> "${tmpdir}/output"
    if [ "$?" -ne 0 ]; then
        cat "${tmpdir}/output" 1>&2
        printf "%s: failed to fetch new feed for '%s'\n" "$0" "${feed}" 1>&2
        return 1
    fi

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

    # Add any new entries.
    LC_ALL="C" comm -23 "${new_entries}" \
        <(cd "${feed}/entry/"; printf '%s\n' * | LC_ALL="C" sort) |
    while IFS="\n" read -r entry; do
        local entry_name
        entry_name="$(feed-unescape "${entry}")"
        if [ "$?" -ne 0 ]; then
            printf "%s: unescaping failed\n" "$0" 1>&2
            return 1
        fi

        # Extract the entry.
        mkdir "${feed}/entry/${entry}" && \
        atom-extract "${entry_name}" < "${new_feed}" \
            > "${feed}/entry/${entry}/entry"
        if [ "$?" -ne 0 ]; then
            printf "%s: extracting entry from %s failed\n" "$0" "${new_feed}" \
                1>&2
            return 1
        fi

        # Extract the entry date/time.
        atom-timestamp < "${feed}/entry/${entry}/entry" \
            > "${feed}/entry/${entry}/timestamp"
        if [ "$?" -ne 0 ]; then
            printf "%s: retrieving the timestamp from %s failed\n" "$0" \
                "${entry}" 1>&2
            return 1
        fi

        # Cache the entry.
        if [ -x "${feed}/cache" ]; then
            atom-exec "${feed}/entry/${entry}/entry" \
                "${feed}/cache" "${feed}/entry/${entry}" \
                > "${tmpdir}/output" 2>&1
            if [ "$?" -ne 0 ]; then
                cat "${tmpdir}/output" 1>&2
                printf "%s: caching entry '%s' failed\n" "$0" "${entry}" 1>&2
            fi
        fi
    done

    # Clean up old entries.
    LC_ALL="C" comm -23 \
        <(cd "${feed}/entry/"; printf '%s\n' * | LC_ALL="C" sort) \
        "${new_entries}" |
    while IFS="\n" read -r entry; do
        [ -f "${feed}/entry/${entry}/read" ] && rm -rf "${feed}/entry/${entry}"
    done
}

# Create our (global) temporary directory.
tmpdir="$(mktemp -d "${TMPDIR:-/tmp}/feed-update.XXXX")"
if [ "$?" -ne 0 ]; then
    printf "%s: failed to create temporary directory\n" "$0" 1>&2
    exit 1
fi
trap "rm -rf '${tmpdir}'" EXIT

if [ "$#" -ne 0 ]; then
    cd "${FEED_DIR}"
    for arg in "$@"; do
        if [ -d "${arg}" ]; then
            # This is a path to a particular feed; open all unread.
            log_update_feed "${arg}"
        else
            printf '%s: no such feed dir %s\n' "$0" "${arg}" 1>&2
        fi
    done
else
    # No feeds given; update all.
    for feed in "${FEED_DIR}/"*; do
        [ -d "${feed}" ] && log_update_feed "${feed}"
    done
fi
