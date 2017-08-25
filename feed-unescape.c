/* feed-unescape.c
 *
 * Print the given escaped string in unescaped form on stdout.
 *
 * See notes.md for a description of the string escaping algorithm.
 *
 * Author:  Alastair Hughes
 * Contact: hobbitalastair at yandex dot com
 */

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>

int unescape(char* data, int length) {
    /* Unescape (in-place) the given string, returning the new length.
     *
     * See notes.md for documentation of the escaping algorithm.
     */

    int new_len = 0;
    bool escaped = false;
    for (int i = 0; i < length; i++) {
        if (!escaped && data[i] == '\\') {
            escaped = true;
        } else {
            char c = data[i];
            if (escaped && c == '0') c = '\0';
            if (escaped && c == '_') c = '/';
            if (escaped && c == '.') c = '.';
            data[new_len] = c;
            new_len++;
            escaped = false;
        }
    }

    return new_len;
}

int main(int argc, char** argv) {
    char* name = __FILE__;
    if (argc > 0) name = argv[0];
    if (argc != 2) {
        fprintf(stderr, "usage: %s <id>\n", name);
        exit(EXIT_FAILURE);
    }
    int len = strlen(argv[1]);
    len = unescape(argv[1], len);
    printf("%.*s", len, argv[1]);
}
