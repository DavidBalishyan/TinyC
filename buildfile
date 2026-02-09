pm = cargo

[group: General]
all:
	makeover --list

[group: Installation]
install:
	cargo install --path .

uninstall:
	cargo uninstall tcc

reinstall: uninstall install

[group: Utility]
clean:
	cargo clean

build-rel:
	cargo build --release

build:
	cargo build

