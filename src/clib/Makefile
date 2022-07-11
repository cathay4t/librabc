LIB_SO_MAIN_VERSION=0
TMPDIR := $(shell mktemp -d)
TOP_SRC_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/../../
DEBUG_RABC_SO_FILE=$(TOP_SRC_DIR)/target/debug/librabc.so
DAEMON_FILE=$(TOP_SRC_DIR)/target/debug/rabcd
CLIB_SO_FULL=librabc.so.0.1.0
CLIB_SO_MAN=librabc.so.0
CLIB_SO_DEV=librabc.so
C_HEADER=rabc.h

CFLAGS =-g -Wall -Wextra
LIBS =-lrabc
CC ?=cc

all: $(DEBUG_RABC_SO_FILE) rabc_test

HEADER_IN = $(TOP_SRC_DIR)/src/clib/rabc.h.in

.PHONY: $(DEBUG_RABC_SO_FILE) $(DAEMON_FILE)

$(DEBUG_RABC_SO_FILE) $(DAEMON_FILE):
	cargo build --workspace

test_lib: $(DEBUG_RABC_SO_FILE)
	install -p -v -D -m755 $(DEBUG_RABC_SO_FILE) \
		$(TMPDIR)/$(CLIB_SO_FULL)
	ln -sfv $(CLIB_SO_FULL) $(TMPDIR)/$(CLIB_SO_MAN)
	ln -sfv $(CLIB_SO_FULL) $(TMPDIR)/$(CLIB_SO_DEV)

test_header: $(C_HEADER)
	install -p -v -D -m644 $(C_HEADER) $(TMPDIR)/$(C_HEADER)

rabc_test: tests/rabc_test.c
	$(CC) $(CFLAGS) -I./ -L$(TOP_SRC_DIR)/target/debug/ \
		-o rabc_test tests/rabc_test.c $(LIBS)

check: rabc_test test_lib test_header
	$(DAEMON_FILE) &
	LD_LIBRARY_PATH=$(TMPDIR) \
		valgrind --trace-children=yes --leak-check=full \
		--error-exitcode=1 \
		./rabc_test 1>/dev/null
	rm -rf $(TMPDIR)
	pkill rabcd

clean:
	rm -f rabc_test.o rabc_test

