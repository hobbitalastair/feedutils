#!/usr/bin/env sh
#
# Add a new feed with the given RSS URL.
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
    printf 'usage: %s <name> <rss>\n' "$0" 1>&2
    exit 1
fi
name="$1"
rss="$2"

cd "${FEED_DIR}"
mkdir "${name}"
cd "${name}"
ln -s ../browser-open.sh ./open
printf '#!/usr/bin/env sh\n' >> fetch
printf "curl -L -o - '%s' | rss2atom\n" "${rss}" >> fetch
chmod +x fetch
feed-update "${name}"
for i in entry/*; do
    touch "$i/read"
done
printf 'Added feed %s\n' "${name}"
