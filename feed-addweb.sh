#!/usr/bin/env sh
#
# Add a new feed from the given HTML URL.
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

if [ "$#" -ne 3 ]; then
    printf 'usage: %s <name> <url> <link-pattern>\n' "$0" 1>&2
    exit 1
fi
name="$1"
url="$2"
link_pattern="$(printf '%s' "$3" | sed 's:\/:\\/:g')"

cd "${FEED_DIR}"
mkdir "${name}"
cd "${name}"
ln -s ../browser-open.sh ./open

cat > fetch << EOF
#!/usr/bin/sh
url='${url}'

curl -A "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.26 (KHTML, like Gecko) Chrome/28.0.1500.52 Safari/537.26" \\
    -L -o - "\${url}" | \\
    html-extract | \\
    sed -n '/${link_pattern}/p' | \\
    sort -u | \\
    links2atom '${name}' "\${url}"
EOF
chmod +x fetch

feed-update "${name}"
for i in entry/*; do
    touch "$i/read"
done
printf 'Added feed %s\n' "${name}"
