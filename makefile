PREFIX := ${DESTDIR}/usr
BINDIR := ${PREFIX}/bin
COMPLETION_DIR_BASH := ${PREFIX}/share/bash-completion
SYSTEMDUSERDIR := ${PREFIX}/lib/systemd/user
LIBS = `pkg-config --libs expat`
CC = gcc
CFLAGS = -Wall -Werror -O2 -g
OBJS = atom-exec atom-extract atom-list atom-timestamp \
       feed-addrss feed-addweb feed-daily \
       feed-delete feed-read feed-unescape feed-update feed-unread \
       rss2atom snow2feed
SYSTEMD = systemd/feed.service systemd/feed.timer
COMPLETION = completion/bash_completion.sh

all: $(OBJS) $(SYSTEMD)

%: %.c
	$(CC) -o $@ $< $(LIBS) $(CFLAGS)

%: %.sh
	cp -f $< $@
	chmod +x $@

install: install_objs install_systemd install_completion
	
install_objs: $(OBJS)
	mkdir -p "${BINDIR}/"
	for obj in ${OBJS}; do \
	    install -m755 "$$obj" "${BINDIR}/"; \
	done

install_systemd: $(SYSTEMD)
	mkdir -p "${SYSTEMDUSERDIR}/"
	for service in ${SYSTEMD}; do \
	    install -m644 "$$service" "${SYSTEMDUSERDIR}/"; \
	done

install_completion: $(COMPLETION)
	mkdir -p "${COMPLETION_DIR_BASH}/completions/"
	for obj in ${OBJS}; do \
	    ln completion/bash_completion.sh \
	        "${COMPLETION_DIR_BASH}/completions/$$obj"; \
	done

clean:
	rm -f $(OBJS)
