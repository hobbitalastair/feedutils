LIBS = `pkg-config --libs expat`
CC = gcc
CFLAGS = -Wall -Werror -O2 -g
OBJS = atom-exec atom-extract atom-list feed-unescape rss2atom

all: $(OBJS)

%: %.c
	$(CC) -o $@ $< $(LIBS) $(CFLAGS)

%: %.sh
	cp -f $< $@
	chmod +x $@

clean:
	rm -f $(OBJS)
