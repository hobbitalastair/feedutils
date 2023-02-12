#!/usr/bin/env sh
#
# Add a new feed with the given Atom URL.
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

if [ "$#" -ne 2 ]; then
    printf 'usage: %s <name> <atom>\n' "$0" 1>&2
    exit 1
fi
name="$1"
atom="$2"

cd "${FEED_DIR}"
mkdir "${name}"
cd "${name}"
ln -s ../open open
printf '#!/usr/bin/env sh\n' >> fetch
printf "exec curl -L -o - '%s'\n" "${atom}" >> fetch
chmod +x fetch
feed-update "${name}"
feed-markasread "${name}"
printf 'Added feed %s\n' "${name}"
