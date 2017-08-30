LIBS = `pkg-config --libs expat`
CC = gcc
CFLAGS = -Wall -Werror -O2 -g
OBJS = atom-exec atom-extract atom-list \
       feed-read feed-unescape feed-update \
       rss2atom snow2feed

all: $(OBJS)

%: %.c
	$(CC) -o $@ $< $(LIBS) $(CFLAGS)

%: %.sh
	cp -f $< $@
	chmod +x $@

clean:
	rm -f $(OBJS)
