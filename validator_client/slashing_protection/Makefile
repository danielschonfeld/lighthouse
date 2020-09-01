TESTS_TAG := 3220e8a35a315a4dc0d878cb0a09d312a2b06996
OUTPUT_DIR := interchange-tests
TARBALL := $(OUTPUT_DIR)-$(TESTS_TAG).tar.gz
ARCHIVE_URL := https://github.com/eth2-clients/slashing-protection-interchange-tests/tarball/$(TESTS_TAG)

$(OUTPUT_DIR): $(TARBALL)
	mkdir $@
	tar --strip-components=1 -xzf $^ -C $@

$(TARBALL):
	wget $(ARCHIVE_URL) -O $@

clean-test-files:
	rm -rf $(OUTPUT_DIR)

clean-archives:
	rm -f $(TARBALL)

clean: clean-test-files clean-archives

.PHONY: clean clean-archives clean-test-files

