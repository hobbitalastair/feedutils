LIBS = `pkg-config --libs expat`
CC = gcc
CFLAGS = -Wall -Werror -O2 -g
OBJS = atom-list rss2atom

all: $(OBJS)

%: %.c
	$(CC) -o $@ $< $(LIBS) $(CFLAGS)

clean:
	rm -f $(OBJS)
