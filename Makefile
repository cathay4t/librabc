ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
VERSION=$(shell cat $(ROOT_DIR)/VERSION)
VERSION_MAJOR=$(shell echo $(VERSION) | cut -f1 -d.)
VERSION_MINOR=$(shell echo $(VERSION) | cut -f2 -d.)
VERSION_MICRO=$(shell echo $(VERSION) | cut -f3 -d.)
CLIB_SO_DEV=librabc.so
CLIB_SO_MAN=$(CLIB_SO_DEV).$(VERSION_MAJOR)
CLIB_SO_FULL=$(CLIB_SO_DEV).$(VERSION)
CLI_EXEC=rabcc
DAEMON_EXEC=rabcd
CLIB_HEADER=src/clib/rabc.h
CLIB_SO_DEV_RELEASE=target/release/$(CLIB_SO_DEV)
CLIB_SO_DEV_DEBUG=target/debug/$(CLIB_SO_DEV)
CLIB_PKG_CONFIG=src/clib/rabc.pc
DAEMON_DEBUG=target/debug/$(DAEMON_EXEC)
DAEMON_RELEASE=target/release/$(DAEMON_EXEC)
PYTHON_MODULE_NAME=rabc
CLI_EXEC_RELEASE=target/release/$(CLI_EXEC)
PREFIX ?= /usr/local

#outdir is used by COPR as well: https://docs.pagure.org/copr.copr/user_documentation.html
outdir ?= $(ROOT_DIR)

CPU_BITS = $(shell getconf LONG_BIT)
ifeq ($(CPU_BITS), 32)
    LIBDIR ?= $(PREFIX)/lib
else
    LIBDIR ?= $(PREFIX)/lib$(CPU_BITS)
endif

INCLUDE_DIR ?= $(PREFIX)/include
PKG_CONFIG_LIBDIR ?= $(LIBDIR)/pkgconfig
MAN_DIR ?= $(PREFIX)/share/man

SKIP_PYTHON_INSTALL ?=0
SKIP_VENDOR_CREATION ?=0
RELEASE ?=0

PYTHON3_SITE_DIR ?=$(shell \
	python3 -c \
		"from distutils.sysconfig import get_python_lib; \
		 print(get_python_lib())")


.PHONY: debug
debug:
	cargo build --all
	ln -sfv $(CLIB_SO_DEV) target/debug/$(CLIB_SO_FULL)
	ln -sfv $(CLIB_SO_DEV) target/debug/$(CLIB_SO_MAN)

$(CLI_EXEC_RELEASE) $(CLIB_SO_DEV_RELEASE) $(DAEMON_RELEASE):
	cargo build --all --release

$(CLIB_SO_DEV_DEBUG) $(DAEMON_DEBUG): debug

clib: $(CLIB_HEADER) $(CLIB_SO_DEV_RELEASE) $(CLIB_PKG_CONFIG)

.PHONY: $(CLIB_HEADER)
$(CLIB_HEADER): $(CLIB_HEADER).in
	cp $(CLIB_HEADER).in $(CLIB_HEADER)
	sed -i -e 's/@_VERSION_MAJOR@/$(VERSION_MAJOR)/' \
		$(CLIB_HEADER)
	sed -i -e 's/@_VERSION_MINOR@/$(VERSION_MINOR)/' \
		$(CLIB_HEADER)
	sed -i -e 's/@_VERSION_MICRO@/$(VERSION_MICRO)/' \
		$(CLIB_HEADER)

.PHONY: $(CLIB_PKG_CONFIG)
$(CLIB_PKG_CONFIG): $(CLIB_PKG_CONFIG).in
	cp $(CLIB_PKG_CONFIG).in $(CLIB_PKG_CONFIG)
	sed -i -e 's|@VERSION@|$(VERSION)|' $(CLIB_PKG_CONFIG)
	sed -i -e 's|@PREFIX@|$(PREFIX)|' $(CLIB_PKG_CONFIG)
	sed -i -e 's|@LIBDIR@|$(LIBDIR)|' $(CLIB_PKG_CONFIG)
	sed -i -e 's|@INCLUDE_DIR@|$(INCLUDE_DIR)|' $(CLIB_PKG_CONFIG)

.PHONY: clib_check
clib_check: $(CLIB_SO_DEV_DEBUG) $(CLIB_HEADER) $(DAEMON_DEBUG)
	$(eval TMPDIR := $(shell mktemp -d))
	cp $(CLIB_SO_DEV_DEBUG) $(TMPDIR)/$(CLIB_SO_FULL)
	ln -sfv $(CLIB_SO_FULL) $(TMPDIR)/$(CLIB_SO_MAN)
	ln -sfv $(CLIB_SO_FULL) $(TMPDIR)/$(CLIB_SO_DEV)
	cp $(CLIB_HEADER) $(TMPDIR)/$(shell basename $(CLIB_HEADER))
	cc -g -Wall -Wextra -L$(TMPDIR) -I$(TMPDIR) -lrabc \
		-o $(TMPDIR)/rabc_test src/clib/tests/rabc_test.c
	$(DAEMON_DEBUG) &
	LD_LIBRARY_PATH=$(TMPDIR) \
		valgrind --trace-children=yes --leak-check=full \
		--error-exitcode=1 \
		$(TMPDIR)/rabc_test 1>/dev/null
	rm -rf $(TMPDIR)
	pkill $(DAEMON_EXEC)

rust_check:
	cargo test -- --show-output;

check: rust_check clib_check

clean:
	- cargo clean
	- rm -f target/debug/$(CLIB_SO_MAN)
	- rm -f target/debug/$(CLIB_SO_FULL)
	- rm -f $(CLIB_HEADER)

install: $(CLI_EXEC_RELEASE) clib $(DAEMON_RELEASE)
	install -p -v -D -m755 $(CLI_EXEC_RELEASE) \
		$(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	install -p -v -D -m755 $(DAEMON_RELEASE) \
		$(DESTDIR)$(PREFIX)/bin/$(DAEMON_EXEC)
	install -p -D -m755 $(CLIB_SO_DEV_RELEASE) \
		$(DESTDIR)$(LIBDIR)/$(CLIB_SO_FULL)
	ln -sfv $(CLIB_SO_FULL) $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MAN)
	ln -sfv $(CLIB_SO_FULL) $(DESTDIR)$(LIBDIR)/$(CLIB_SO_DEV)
	if [ $(SKIP_PYTHON_INSTALL) != 1 ];then \
		cd src/python; python3 setup.py install; \
	fi
	install -p -v -D -m644 $(CLIB_HEADER) \
		$(DESTDIR)$(INCLUDE_DIR)/$(shell basename $(CLIB_HEADER))
	install -p -v -D -m644 $(CLIB_PKG_CONFIG) \
		$(DESTDIR)$(PKG_CONFIG_LIBDIR)/$(shell basename $(CLIB_PKG_CONFIG))

uninstall:
	- rm -fv $(DESTDIR)$(PREFIX)/bin/$(CLI_EXEC)
	- rm -fv $(DESTDIR)$(PREFIX)/bin/$(DAEMON_EXEC)
	- rm -fv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_DEV)
	- rm -fv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_MAN)
	- rm -fv $(DESTDIR)$(LIBDIR)/$(CLIB_SO_FULL)
	- rm -fv $(DESTDIR)$(INCLUDE_DIR)/$(shell basename $(CLIB_HEADER))
	- rm -fv $(DESTDIR)$(INCLUDE_DIR)/$(shell basename $(CLIB_PKG_CONFIG))
	- if [ $(SKIP_PYTHON_INSTALL) != 1 ];then \
		rm -rfv $(DESTDIR)$(PYTHON3_SITE_DIR)/$(PYTHON_MODULE_NAME); \
	fi
