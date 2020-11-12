SUBDIRS := $(wildcard ./module_examples/modules/*)

all: $(SUBDIRS)

test: $(SUBDIRS)
	cargo test

$(SUBDIRS):
	$(MAKE) -C $@

.PHONY: all $(SUBDIRS)
