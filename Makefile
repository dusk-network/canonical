SUBDIRS := $(wildcard canon_host/examples/*)

all: $(SUBDIRS)

test: $(SUBDIRS)
	cargo test

$(SUBDIRS):
	$(MAKE) -C $@

.PHONY: all $(SUBDIRS)
