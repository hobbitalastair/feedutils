/* rss2atom.c
 *
 * Convert an RSS feed to the Atom feed format.
 *
 * The RSS feed to be converted is assumed to be given on stdin, and the
 * resulting Atom feed is printed to stdout.
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

#define READBUF_SIZE 4096 /* Size of the read buffer, in bytes */
#define DATABUF_SIZE 4096 /* Size of the persistent string buffer */
#define FIELD_SIZE 100 /* Maximum field length */

#ifdef XML_LARGE_SIZE
#define XML_FMT_INT_MOD "ll"
#else
#define XML_FMT_INT_MOD "l"
#endif

char* name; /* Program name */

typedef struct {
    char* title;
    char* link;
    char* description;
    char* author;
    char* lastBuildDate;
    char* category;
    char* copyright;
    char* generator;
    char* managingEditor;
    char* pubDate;
} RSS_Channel;

typedef struct {
    char* title;
    char* link;
    char* description;
    char* author;
    char* category;
    char* guid;
    char* pubDate;
} RSS_Item;

enum RSS_Elements {
    RSS_TYPE_NONE,
    RSS_TYPE_RSS,
    RSS_TYPE_CHANNEL,
    RSS_TYPE_ITEM,
};

typedef struct {
    /* Current main element.
     *
     * Because RSS documents have a (fairly) structured form we can use a
     * single union element for storing the current "midlevel" item.
     *
     * Some RSS specifications let a channel be a seperate section, while
     * others put the items inside the channel. This complicates things
     * somewhat since now the channel may need to be written out early.
     */
    enum RSS_Elements element;
    union {
        RSS_Channel channel;
        RSS_Item item;
    } v;
    bool have_channel; /* True if we have previously printed the feed header */

    /* Current field element.
     *
     * We store the name for tag end handling and a pointer to the actual
     * field, since we need to track which field we are manipulating but don't
     * want to have to check the field name and current element each time we go
     * to access it.
     */
    char field[FIELD_SIZE];
    char** field_ptr;

    /* String buffer used for storing the data from elements.
     * Because only the last element can ever have data appended, we can keep
     * a global buffer and store pointers into it instead of allocating memory
     * for each new string.
     *
     * This does, however, put a hard limit on the amount of data that each
     * channel, image, and item can store in the various fields.
     *
     * TODO: Is the hard limit a problem? If so, investigate using a
     *       dynamically allocated buffer.
     */
    ssize_t offset;
    char databuf[DATABUF_SIZE];

    /* Current depth in unknown tags.
     *
     * We store a depth of unknown tags to allow us to ignore tags that we do
     * recognize nested in a tag that we don't.
     */
    int unknown_depth;
} State;


void print_escaped(char* string, bool attribute) {
    /* Print the given string, escaped to avoid interfering with the rest of the
     * xml output. If 'attribute', use rules for attribute escaping, otherwise
     * just use rules for content escaping.
     *
     * This prints the given string, escaped using the rules described in
     * `recycledknowledge.blogspot.co.nz/2006/03/writing-out-xml.html`.
     * A quick search did not find any other sources of information regarding
     * the escaping rules required, so I'll be using these for now.
     *
     * Note that I'm assuming a UTF-8 or UTF-16 encoding here, which probably
     * should be checked.
     *
     * TODO: Confirm that this behaves as expected.
     */

    while (*string != '\0') {
        if (*string == '&') {
            printf("&amp;");
        } else if (*string == '<') {
            printf("&lt;");
        } else if (*string == '>') {
            printf("&gt;");
        } else if (*string == '\r') {
            printf("&#xD;");
        } else if (attribute && *string == '\t') {
            printf("&#x9;");
        } else if (attribute && *string == '\n') {
            printf("&#xA;");
        } else {
            putchar(*string);
        }
        string++;
    }
}

void print_updated(char* rss_datetime) {
    /* Print the time of the last update, using the given rss_datetime if
     * nonnull, or otherwise the current time.
     *
     * We need a "last updated" field entry in an atom-friendly format
     * (ISO.8601.1988). Unfortunately specifying the date/time is slightly
     * tricky, so we cheat here and just use a placeholder.
     * RSS uses a different date/time format (RFC-822), so we would need to
     * parse that and convert it to something atom-friendly if we wanted to
     * properly use the RSS-provided fields.
     * use the lastBuildDate or pubDate fields properly.
     *
     * TODO: Implement proper date/time parsing and usage.
     */
    if (rss_datetime == NULL) rss_datetime = "placeholder date/time";
    printf("\t\t<updated>");
    print_escaped(rss_datetime, false);
    printf("</updated>\n");
}

void print_id(char* id) {
    /* Print the id.
     *
     * Note that the Atom specification has a "normalization strategy" for
     * ensuring that the ids are universal; we ignore that here and hope that
     * it is sufficient.
     */
    printf("\t\t<id>");
    print_escaped(id, false);
    printf("</id>\n");
}

void print_link(char* link) {
    /* Print the link - this should be a IRI */
    printf("\t\t<link href=\"");
    print_escaped(link, true);
    printf("\"></link>\n");
}

void print_category(char* category) {
    /* Print the category, if non-NULL */
    if (category != NULL) {
        printf("\t\t<category term=\"");
        print_escaped(category, true);
        printf("\"></category>\n");
    }
}

void print_channel(RSS_Channel* channel) {
    /* Print out the given channel to stdout */

    printf("<feed xmlns=\"http://www.w3.org/2005/Atom\">\n");

    char* title = channel->title;
    if (title == NULL) {
        fprintf(stderr, "%s: malformed feed: no channel title\n", name);
        exit(EXIT_FAILURE);
    }
    printf("\t\t<title>");
    print_escaped(title, false);
    printf("</title>\n");

    if (channel->description != NULL) {
        printf("\t\t<subtitle>\n");
        print_escaped(channel->description, false);
        printf("\n\t\t</subtitle>\n");
    }

    /* We cheat here and just use the link provided in the channel for an id.
     *
     * Not all RSS feeds have a link (despite it being specified in the spec)
     * so failing that we just use the title.
     */
    char* id = channel->link;
    if (id == NULL) {
        id = channel->title;
        fprintf(stderr, "%s: malformed feed: no channel link\n", name);
    }
    print_id(id);
    print_link(id);

    /* Technically we don't need a toplevel author if all the entries specify
     * an author. However we can't check that right now, so just specify a
     * placeholder if we aren't given one.
     */
    char* author = channel->author;
    if (author == NULL) author = channel->managingEditor;
    if (author == NULL) author = "Unknown Author";
    printf("\t\t<author><name>");
    print_escaped(author, false);
    printf("</name></author>\n");

    char* updated = channel->pubDate;
    if (updated == NULL) updated = channel->lastBuildDate;
    print_updated(updated);

    print_category(channel->category);

    if (channel->copyright != NULL) {
        printf("\t\t<rights>");
        print_escaped(channel->copyright, false);
        printf("</rights>\n");
    }

    if (channel->generator != NULL) {
        printf("\t\t<generator>");
        print_escaped(channel->generator, false);
        printf("</generator>\n");
    }
}

void print_item(RSS_Item* item) {
    /* Print out the given item to stdout */

    printf("\t<entry>\n");

    char* title = item->title;
    if (title == NULL) {
        fprintf(stderr, "%s: malformed feed: no item title\n", name);
        exit(EXIT_FAILURE);
    }
    printf("\t\t<title>");
    print_escaped(title, false);
    printf("</title>\n");

    if (item->description != NULL) {
        printf("\t\t<content>\n");
        print_escaped(item->description, false);
        printf("\n\t\t</content>\n");
    }

    /* We cheat here and just use the link provided in the item.
     *
     * If there is no link provided, we fall back to the GUID (which isn't
     * technically valid, but oh well) and failing that fall back to the
     * title.
     */
    char* id = item->link;
    if (id == NULL) {
        id = item->guid;
        fprintf(stderr, "%s: malformed feed: no item link\n", name);
    }
    if (id == NULL) id = item->title;
    print_id(id);
    print_link(id);

    char* author = item->author;
    if (author == NULL) author = "Unknown Author";
    printf("\t\t<author><name>");
    print_escaped(author, false);
    printf("</name></author>\n");

    print_updated(item->pubDate);

    print_category(item->category);

    printf("\t</entry>\n");
}

void start_handler(void* data, const char* element, const char** attributes) {
    /* Handle an element start tag */
    State* s = (State*)data;

    if (s->unknown_depth > 0) {
        s->unknown_depth++;
        return;
    }

    if (s->field[0] != '\0') {
        /* We don't support nested field tags */
        goto unhandled_tag;
    } else if (strcmp("item", element) == 0) {
        if (s->element != RSS_TYPE_RSS && s->element != RSS_TYPE_CHANNEL) {
            fprintf(stderr, "%s: malformed feed: unexpected item when not in"
                    " RSS or CHANNEL\n", name);
            exit(EXIT_FAILURE);
        }
        if (s->element == RSS_TYPE_CHANNEL) {
            /* We print the feed header since we need it before any entries */
            print_channel(&(s->v.channel));
            s->have_channel = true;
        }
        if (!s->have_channel) {
            /* We need to check that we have printed the feed header before
             * we can print an item.
             */
            fprintf(stderr,
                    "%s: malformed feed: no channel entry before item\n",
                    name);
            exit(EXIT_FAILURE);
        }
        s->element = RSS_TYPE_ITEM;
        s->offset = 0;
        memset(&(s->v.item), 0, sizeof(RSS_Item));
    } else if (strcmp("channel", element) == 0) {
        if (s->element != RSS_TYPE_RSS) {
            fprintf(stderr, "%s: malformed feed: unexpected channel when not"
                    " in RSS\n", name);
            exit(EXIT_FAILURE);
        }
        s->element = RSS_TYPE_CHANNEL;
        s->offset = 0;
        memset(&(s->v.channel), 0, sizeof(RSS_Channel));
    } else if (strcmp("rss", element) == 0 ||
               strcmp("rdf:RDF", element) == 0) {
        if (s->element != RSS_TYPE_NONE) {
            fprintf(stderr, "%s: malformed feed: unexpected rss when not at"
                    " document root\n", name);
            exit(EXIT_FAILURE);
        }
        s->element = RSS_TYPE_RSS;
        s->have_channel = false;
    } else if (s->element == RSS_TYPE_ITEM || s->element == RSS_TYPE_CHANNEL) {
        /* Handle a new potential field tag */

        s->field_ptr = NULL;
        if (s->element == RSS_TYPE_ITEM) {
            if (strcmp("title", element) == 0) {
                s->field_ptr = &(s->v.item.title);
            } else if (strcmp("link", element) == 0) {
                s->field_ptr = &(s->v.item.link);
            } else if (strcmp("description", element) == 0) {
                s->field_ptr = &(s->v.item.description);
            } else if (strcmp("author", element) == 0) {
                s->field_ptr = &(s->v.item.author);
            } else if (strcmp("category", element) == 0) {
                s->field_ptr = &(s->v.item.category);
            } else if (strcmp("guid", element) == 0) {
                s->field_ptr = &(s->v.item.guid);
            } else if (strcmp("pubDate", element) == 0) {
                s->field_ptr = &(s->v.item.pubDate);
            }
        } else if (s->element == RSS_TYPE_CHANNEL) {
            if (strcmp("title", element) == 0) {
                s->field_ptr = &(s->v.channel.title);
            } else if (strcmp("link", element) == 0) {
                s->field_ptr = &(s->v.channel.link);
            } else if (strcmp("description", element) == 0) {
                s->field_ptr = &(s->v.channel.description);
            } else if (strcmp("author", element) == 0) {
                s->field_ptr = &(s->v.channel.author);
            } else if (strcmp("lastBuildDate", element) == 0) {
                s->field_ptr = &(s->v.channel.lastBuildDate);
            } else if (strcmp("category", element) == 0) {
                s->field_ptr = &(s->v.channel.category);
            } else if (strcmp("copyright", element) == 0) {
                s->field_ptr = &(s->v.channel.copyright);
            } else if (strcmp("generator", element) == 0) {
                s->field_ptr = &(s->v.channel.generator);
            } else if (strcmp("managingEditor", element) == 0) {
                s->field_ptr = &(s->v.channel.managingEditor);
            } else if (strcmp("pubDate", element) == 0) {
                s->field_ptr = &(s->v.channel.pubDate);
            }
        }

        if (s->field_ptr == NULL) {
            goto unhandled_tag;
        }
        strncpy(s->field, element, FIELD_SIZE);
    } else {
        goto unhandled_tag;
    }

    return; /* Default exit */
unhandled_tag:
    s->unknown_depth++;
    fprintf(stderr, "%s: unhandled tag: %s\n", name, element);
}

void end_handler(void* data, const char* element) {
    /* Handle an element end tag */
    State* s = (State*)data;

    if (s->unknown_depth > 0) {
        s->unknown_depth--;
        return;
    }

    if (s->field[0] != '\0') {
        /* We are currently parsing a field */
        if (strncmp(s->field, element, FIELD_SIZE) == 0) {
            s->databuf[s->offset] = '\0';
            s->offset++;
            s->field[0] = '\0';
            s->field_ptr = NULL;
        } else {
            fprintf(stderr, "%s: unhandled end tag when parsing field: %s\n",
                    name, element);
        }
    } else if ((strcmp("rss", element) == 0 ||
                strcmp("rdf:RDF", element) == 0) &&
            s->element == RSS_TYPE_RSS) {
        printf("</feed>\n");
        s->element = RSS_TYPE_NONE;
        s->have_channel = false;
    } else if (strcmp("channel", element) == 0 && 
            (s->element == RSS_TYPE_RSS || s->element == RSS_TYPE_CHANNEL)) {
        if (!s->have_channel) {
            print_channel(&(s->v.channel));
            s->have_channel = true;
        }
        s->element = RSS_TYPE_RSS;
    } else if (strcmp("item", element) == 0 && s->element == RSS_TYPE_ITEM) {
        print_item(&(s->v.item));
        s->element = RSS_TYPE_RSS;
    } else {
        fprintf(stderr, "%s: unhandled end tag: %s\n", name, element);
    }
}

void data_handler(void* data, const char* contents, int len) {
    /* Handle some textual data inside a tag.
     *
     * Note that the data may be given to us in chunks - worst case scenario,
     * several calls may be made to this function for a single piece of textual
     * data.
     */
    State* s = (State*)data;

    if (s->unknown_depth == 0 && s->field[0] != '\0') {
        /* We are currently inside some kind of field - store the data */
        if (*(s->field_ptr) == NULL) {
            /* Allocate a new region */
            *(s->field_ptr) = &(s->databuf[s->offset]);
        }

        /* Copy the data to the old offset and update the offset pointer.
         * Note that we need to leave space to null-terminate the data!
         */
        if (s->offset + len < DATABUF_SIZE) {
            memcpy(&(s->databuf[s->offset]), contents, len);
            s->offset += len;
        } else {
            fprintf(stderr, "%s: malformed feed: too much data\n", name);
            exit(EXIT_FAILURE);
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

    State s;
    s.element = RSS_TYPE_NONE;
    s.have_channel = false;
    s.field[0] = '\0';
    s.field_ptr = NULL;

    /* Create the parser for the feed */
    XML_Parser parser = XML_ParserCreate(NULL);
    if (parser == NULL) {
        fprintf(stderr, "%s: failed to create XML parser\n", name);
        exit(EXIT_FAILURE);
    }
    XML_SetUserData(parser, &s);
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
