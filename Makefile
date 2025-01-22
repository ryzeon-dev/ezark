main:
	cargo build -r

install:
	cp ./target/release/ezark /usr/local/bin/