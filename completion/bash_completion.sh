_feed_list()
{
    [ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"
    [ ! -d "${FEED_DIR}" ] && return 1

    local cur prev OPTS
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    local IFS=$'\n'
    compopt -o filenames
    COMPREPLY=( $(compgen -W "$(printf '%s\n' "${FEED_DIR}"/*/ | rev | cut -d/ -f2 | rev)" -- $cur) )
    return 0
}

_feed_list_unread()
{
    [ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"
    [ ! -d "${FEED_DIR}" ] && return 1

    local cur prev OPTS
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    local IFS=$'\n'
    compopt -o filenames
    COMPREPLY=( $(compgen -W "$(for entry in "${FEED_DIR}"/*/entry/*; do [ ! -f "${entry}/read" ] && printf '%s\n' "${entry}" | rev | cut -d/ -f3 | rev; done)" -- $cur) )
    return 0
}

complete -F _feed_list feed-delete
complete -F _feed_list_unread feed-read
complete -F _feed_list feed-update
