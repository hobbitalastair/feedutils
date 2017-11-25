PREFIX := ${DESTDIR}/usr
BINDIR := ${PREFIX}/bin
SYSTEMDUSERDIR := ${PREFIX}/lib/systemd/user
LIBS = `pkg-config --libs expat`
CC = gcc
CFLAGS = -Wall -Werror -O2 -g
OBJS = atom-exec atom-extract atom-list atom-timestamp \
       feed-addrss feed-read feed-unescape feed-update \
       rss2atom snow2feed
SYSTEMD = systemd/feed.service systemd/feed.timer

all: $(OBJS) $(SYSTEMD)

%: %.c
	$(CC) -o $@ $< $(LIBS) $(CFLAGS)

%: %.sh
	cp -f $< $@
	chmod +x $@

install: $(OBJS) $(SYSTEMD)
	mkdir -p "${BINDIR}/"
	for obj in ${OBJS}; do \
	    install -m755 "$$obj" "${BINDIR}/"; \
	done
	mkdir -p "${SYSTEMDUSERDIR}/"
	for service in ${SYSTEMD}; do \
	    install -m644 "$$service" "${SYSTEMDUSERDIR}/"; \
	done

clean:
	rm -f $(OBJS)
