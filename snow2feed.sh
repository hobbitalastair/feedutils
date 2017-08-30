#!/usr/bin/env sh
#
# Import the given feed file (in `snownews` format).
#
# Author:   Alastair Hughes
# Contact:  hobbitalastair at yandex dot com

[ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"

if [ "$#" -ne 1 ]; then
    printf 'usage: %s urls\n' "$0"
    exit 1
fi

while IFS='|' read -r url name category filter; do
    printf "%s: exporting feed '%s'\n" "$0" "${name}"
    name="$(printf "%s" "${name}" | tr ' ' '_')"

    mkdir "${FEED_DIR}/${name}"
    if [ "$?" -ne 0 ]; then
        printf "%s: failed to create dir with name '%s'\n" "$0" "${name}"
        exit 1
    fi

    if [ "$(printf "%s" "${url}" | cut -d ':' -f 1)" == "exec" ]; then
        fetch="$(printf "%s" "${url}" | cut -d ':' -f 2-)"
    else
        fetch="curl -L -o - '${url}'"
    fi
    
    if [ -n "${filter}" ]; then
        if printf "%s" "${filter}" | grep -e 'atom2rss' > /dev/null; then
            filter=""
        else
            filter=" | ${filter} | rss2atom"
        fi
    else
        filter=" | rss2atom"
    fi

    cat > "${FEED_DIR}/${name}/fetch" << EOF
#!/usr/bin/env sh
${fetch} ${filter}
EOF
    chmod +x "${FEED_DIR}/${name}/fetch"

    ln -s "../open" "${FEED_DIR}/${name}/open"
done < "$1"
