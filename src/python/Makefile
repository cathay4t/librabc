LIB_SO_MAIN_VERSION=0
TMPDIR := $(shell mktemp -d)
TOP_SRC_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))/../../
DEBUG_RABC_SO_FILE=$(TOP_SRC_DIR)/target/debug/librabc.so
DAEMON_FILE=$(TOP_SRC_DIR)/target/debug/rabcd
CLIB_SO_FULL=librabc.so.0.1.0
CLIB_SO_MAN=librabc.so.0
CLIB_SO_DEV=librabc.so

all: $(DEBUG_RABC_SO_FILE) check

.PHONY: $(DEBUG_RABC_SO_FILE) $(DAEMON_FILE)

$(DEBUG_RABC_SO_FILE) $(DAEMON_FILE):
	cargo build --workspace

test_lib: $(DEBUG_RABC_SO_FILE)
	install -p -v -D -m755 $(DEBUG_RABC_SO_FILE) \
		$(TMPDIR)/$(CLIB_SO_FULL)
	ln -sfv $(CLIB_SO_FULL) $(TMPDIR)/$(CLIB_SO_MAN)
	ln -sfv $(CLIB_SO_FULL) $(TMPDIR)/$(CLIB_SO_DEV)

check: test_lib
	- pkill rabcd
	$(DAEMON_FILE) &
	LD_LIBRARY_PATH=$(TMPDIR) \
	PYTHONPATH=$(TOP_SRC_DIR)/src/python \
		python3 test/rabc_test.py
	rm -rf $(TMPDIR)
	pkill rabcd
