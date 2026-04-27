build:
	cargo build
test: build
	./src/test.sh
	rm -f ./tmp ./tmp.s

.PHONY: build test


