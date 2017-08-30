/* atom-list.c
 *
 * List the id of each entry in an Atom feed fed into stdin.
 *
 * The id of each entry is escaped to make the resulting string safe to use for
 * a filename on a UNIX filesystem, and each id is null-terminated.
 *
 * Author:  Alastair Hughes
 * Contact: hobbitalastair at yandex dot com
 */

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>

#include <expat.h>

#define READBUF_SIZE 4096 /* Size of the read buffer, in bytes */
#define DATABUF_SIZE 4096 /* Size of the persistent string buffer */

#ifdef XML_LARGE_SIZE
#define XML_FMT_INT_MOD "ll"
#else
#define XML_FMT_INT_MOD "l"
#endif

char* name; /* Program name */

typedef struct {
    bool is_entry;
    bool is_id;

    int len;
    char data[DATABUF_SIZE];
} Feed;

void print_id(char* data, int length) {
    /* Print the id, escaped so that it can be used in a filesystem, with '\0'
     * as the separator.
     *
     * We assume UNIX filesystem semantics - this disallows forward slashes
     * and nulls. However, we also need to be careful to avoid a '.' or '..';
     * to avoid this we also escape the first character if it is a '.'.
     * We also disallow empty ids.
     *
     * See notes.md for documentation of the escaping algorithm.
     */

    if (length == 0) {
        fprintf(stderr, "%s: invalid empty id\n", name);
        exit(EXIT_FAILURE);
    }

    if (data[0] == '.') putc('\\', stdout);

    for (int i = 0; i < length; i++) {
        char c = data[i];
        if (c == '\\') {
            putc('\\', stdout);
            putc('\\', stdout);
        } else if (c == '\0') {
            putc('\\', stdout);
            putc('0', stdout);
        } else if (c == '/') {
            putc('\\', stdout);
            putc('_', stdout);
        } else if (c == '\n') {
            putc('\\', stdout);
            putc('n', stdout);
        } else {
            putc(c, stdout);
        }
    }
    putc('\0', stdout); /* NULL-terminate the strings */
}

void start_handler(void* data, const char* element, const char** attributes) {
    /* Handle an element start tag */
    Feed* feed = (Feed*)data;

    if (feed->is_entry && strcmp("id", element) == 0) {
        feed->is_id = true;
    } else if (strcmp("entry", element) == 0) {
        feed->is_entry = true;
    }
}

void end_handler(void* data, const char* element) {
    /* Handle an element end tag */
    Feed* feed = (Feed*)data;

    if (feed->is_id && strcmp("id", element) == 0) {
        feed->is_id = false;
        print_id(feed->data, feed->len);
        feed->len = 0;
    }
    if (feed->is_entry && strcmp("entry", element) == 0) {
        feed->is_id = false;
        feed->is_entry = false;
    }
}

void data_handler(void* data, const char* contents, int len) {
    /* Handle some textual data inside a tag.
     *
     * Note that the data may be given to us in chunks - worst case scenario,
     * several calls may be made to this function for a single piece of textual
     * data.
     */
    Feed* feed = (Feed*)data;

    if (feed->is_id) {
        if (feed->len + len < DATABUF_SIZE) {
            memcpy(&feed->data[feed->len], contents, len);
            feed->len += len;
        }
    }
}

int main(int argc, char** argv) {
    name = __FILE__;
    if (argc > 0) name = argv[0];
    if (argc != 1) {
        fprintf(stderr, "usage: %s\n", name);
        exit(EXIT_FAILURE);
    }

    Feed feed;
    feed.is_entry = false;
    feed.is_id = false;

    /* Create the parser for the feed */
    XML_Parser parser = XML_ParserCreate(NULL);
    if (parser == NULL) {
        fprintf(stderr, "%s: failed to create XML parser\n", name);
        exit(EXIT_FAILURE);
    }
    XML_SetUserData(parser, &feed);
    XML_SetElementHandler(parser, start_handler, end_handler);
    XML_SetCharacterDataHandler(parser, data_handler);

    /* Feed the data to the parser */
    char buf[READBUF_SIZE];
    ssize_t count = 1; /* Bytes read */
    while (count != 0 || (count == -1 && errno == EINTR)) {
        count = read(0, buf, sizeof(buf));
        if (count < 0) continue; /* Bail on read error */
        if (XML_Parse(parser, buf, count, count == 0) == XML_STATUS_ERROR) {
            fprintf(stderr, "%s: %s at %" XML_FMT_INT_MOD "u\n", name,
                    XML_ErrorString(XML_GetErrorCode(parser)),
                    XML_GetCurrentLineNumber(parser));
            exit(EXIT_FAILURE);
        }
    }
    if (count == -1) {
        fprintf(stderr, "%s: read(): %s\n", name, strerror(errno));
        exit(EXIT_FAILURE);
    }
}
