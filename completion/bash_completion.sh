_feed_list()
{
    [ -z "${FEED_DIR}" ] && FEED_DIR="${XDG_CONFIG_DIR:-${HOME}/.config}/feeds/"
    [ ! -d "${FEED_DIR}" ] && return 1

    local cur prev OPTS
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    local IFS=$'\n'
    compopt -o filenames
    COMPREPLY=( $(compgen -W "$( \
        shopt -s nullglob; \
        printf '%s\n' "${FEED_DIR}"/*/ | rev | cut -d/ -f2 | rev)" -- $cur) )
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
    COMPREPLY=( $(compgen -W "$(
        feed-unread | sed "s/[ ]*[0-9][0-9]* \($cur\)/\1/"
        )" -- $cur) )
    return 0
}

complete -F _feed_list feed-delete
complete -F _feed_list_unread feed-read
complete -F _feed_list feed-update
complete -F _feed_list feed-markasread
