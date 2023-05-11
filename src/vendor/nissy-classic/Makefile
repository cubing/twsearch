# See LICENSE file for copyright and license details.

VERSION = 2.0.4

PREFIX = /usr/local
MANPREFIX = ${PREFIX}/share/man

CPPFLAGS = -DVERSION=\"${VERSION}\"
CFLAGS   = -std=c99 -pthread -pedantic -Wall -Wextra \
	   -Wno-unused-parameter -O3 ${CPPFLAGS}
DBGFLAGS = -std=c99 -pthread -pedantic -Wall -Wextra \
           -Wno-unused-parameter -Wno-unused-function \
	   -fsanitize=address -fsanitize=undefined \
	   -g3 ${CPPFLAGS}

CC = cc


all: nissy

nissy: clean
	${CC} ${CFLAGS} -o nissy src/*.c

nissy.exe:
	x86_64-w64-mingw32-gcc ${CFLAGS} -static -o nissy.exe src/*.c

debug:
	${CC} ${DBGFLAGS} -o nissy src/*.c

clean:
	rm -rf nissy nissy*.exe nissy*.tar.gz

dist: clean nissy.exe
	mkdir -p nissy-${VERSION}
	cp -R LICENSE Makefile INSTALL doc src nissy-${VERSION}
	groff -Tpdf   -mandoc doc/nissy.1 > doc/nissy.pdf 
	groff -Thtml  -mandoc doc/nissy.1 > doc/nissy.html 
	cp doc/nissy.pdf nissy-${VERSION}/doc/nissy.pdf 
	cp doc/nissy.html nissy-${VERSION}/doc/nissy.html 
	tar -cf nissy-${VERSION}.tar nissy-${VERSION}
	gzip nissy-${VERSION}.tar
	rm -rf nissy-${VERSION}
	mv nissy.exe nissy-${VERSION}.exe

upload: dist
	rsync -v --rsync-path=openrsync nissy-${VERSION}.exe \
		tronto.net:/var/www/htdocs/nissy.tronto.net/
	rsync -v --rsync-path=openrsync nissy-${VERSION}.tar.gz \
		tronto.net:/var/www/htdocs/nissy.tronto.net/

website:
	rsync -rv --rsync-path=openrsync \
		www/ tronto.net:/var/www/htdocs/nissy.tronto.net

install: nissy
	mkdir -p ${DESTDIR}${PREFIX}/bin
	cp -f nissy ${DESTDIR}${PREFIX}/bin/nissy
	chmod 755 ${DESTDIR}${PREFIX}/bin/nissy
	mkdir -p ${DESTDIR}${MANPREFIX}/man1
	sed "s/VERSION/${VERSION}/g" < doc/nissy.1 \
				     > ${DESTDIR}${MANPREFIX}/man1/nissy.1
	chmod 644 ${DESTDIR}${MANPREFIX}/man1/nissy.1

uninstall:
	rm -rf ${DESTDIR}${PREFIX}/bin/nissy ${DESTDIR}${MANPREFIX}/man1/nissy.1
	for s in ${SCRIPTS}; do rm -rf ${DESTDIR}${PREFIX}/bin/$$s; done

.PHONY: all debug clean dist install uninstall upload

