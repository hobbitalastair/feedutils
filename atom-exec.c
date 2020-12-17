/* atom-exec.c
 *
 * Execute the given program with TITLE, LINK, CONTENTS, and UPDATED set to
 * the values of the corresponding tags in an Atom entry fed into stdin.
 *
 * Note that tags with a null byte in the contents will have a truncated value
 * set as a null byte is used for the string terminator.
 *
 * Author:  Alastair Hughes
 * Contact: hobbitalastair at yandex dot com
 */

#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>
#include <unistd.h>
#include <errno.h>

#include <expat.h>

#include "config.h"

#ifdef XML_LARGE_SIZE
#define XML_FMT_INT_MOD "ll"
#else
#define XML_FMT_INT_MOD "l"
#endif

char* name; /* Program name */

typedef enum {
    ATOM_NONE,
    ATOM_TITLE,
    ATOM_LINK,
    ATOM_CONTENT,
    ATOM_UPDATED
} Tag;
char* tag_names[] = {NULL, "TITLE", "LINK", "CONTENT", "UPDATED"};

typedef struct {
    Tag tag;
    int len;
    char data[DATABUF_SIZE];
} Feed;

bool attribute_is(const char* name, const char* attribute) {
    /* Return true if the given attribute name matches the given name, minus
     * any namespace prefix.
     */

    const char* name_start = attribute;
    while (*attribute != '\0') {
        if (*attribute == ':') name_start = attribute + 1;
        attribute++;
    }

    return strcmp(name, name_start) == 0;
}

void start_handler(void* data, const char* element, const char** attributes) {
    /* Handle an element start tag */
    Feed* feed = (Feed*)data;

    if (feed->tag != ATOM_NONE) {
        fprintf(stderr, "%s: malformed feed: unexpected tag '%s'\n",
                name, element);
        exit(1);
    }

    feed->len = 0;

    if (strcmp("title", element) == 0) feed->tag = ATOM_TITLE;
    if (strcmp("link", element) == 0) {
        /* Extract the attributes that we care about */
        const char* href = NULL;
        const char* rel = NULL;
        while (*attributes != NULL) {
            if (attribute_is("href", *attributes)) href = *(attributes+1);
            if (attribute_is("rel", *attributes)) rel = *(attributes+1);
            attributes += 2;
        }

        if (href == NULL) {
            fprintf(stderr, "%s: malformed feed: link with no href\n", name);
        } else if (rel == NULL || strcmp(rel, "alternate") == 0) {
            /* We only care about rel="alternate" links.
             *
             * The Atom spec indicates that if no rel is provided, we should treat
             * the link as having rel="alternate".
             * If a different rel is in place, we just ignore the entry (it is
             * probably a comment feed or similar).
             */
            feed->tag = ATOM_LINK;
            /* Link elements use the href attribute to store the actual link */
            feed->len = strlen(href);
            if (feed->len >= DATABUF_SIZE) {
                fprintf(stderr, "%s: malformed feed: link too large\n", name);
                exit(EXIT_FAILURE);
            }
            strncpy(feed->data, href, DATABUF_SIZE - 1);
            feed->data[DATABUF_SIZE - 1] = '\0';
        }
    }
    if (strcmp("content", element) == 0) feed->tag = ATOM_CONTENT;
    if (strcmp("updated", element) == 0) feed->tag = ATOM_UPDATED;
}

void end_handler(void* data, const char* element) {
    /* Handle an element end tag */
    Feed* feed = (Feed*)data;

    feed->data[feed->len] = '\0';
    if (feed->tag != ATOM_NONE) {
        if (setenv(tag_names[feed->tag], feed->data, 1) == -1) {
            fprintf(stderr, "%s: setenv(%s): %s\n", name,
                    feed->data, strerror(errno));
            exit(1);
        }
        feed->tag = ATOM_NONE;
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

    if (feed->tag != ATOM_NONE) {
        if (feed->len + len < DATABUF_SIZE) {
            memcpy(&feed->data[feed->len], contents, len);
            feed->len += len;
        }
    }
}

int main(int argc, char** argv) {
    name = __FILE__;
    if (argc > 0) name = argv[0];
    if (argc <= 2) {
        fprintf(stderr, "usage: %s <file> <child>\n", name);
        exit(EXIT_FAILURE);
    }

    Feed feed = {0};
    feed.tag = ATOM_NONE;

    /* Open the file */
    int fd = open(argv[1], 0);
    if (fd < 0) {
        fprintf(stderr, "%s: open(%s): %s\n", name, argv[1], strerror(errno));
        exit(EXIT_FAILURE);
    }

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
        count = read(fd, buf, sizeof(buf));
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

    /* Run the child */
    for (int i = 0; i < argc - 2; i++) {
        argv[i] = argv[i + 2];
    }
    argv[argc - 2] = NULL;
    close(fd);
    execv(argv[0], argv);
    fprintf(stderr, "%s: exec(): %s\n", name, strerror(errno));
    exit(EXIT_FAILURE);
}
