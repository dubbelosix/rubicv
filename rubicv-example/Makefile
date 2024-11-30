TARGET=riscv32im-unknown-none-elf
OUTPUT_DIR=target/$(TARGET)/release
OUTPUT_BIN=$(OUTPUT_DIR)/output.bin
RUST_BINARY=$(OUTPUT_DIR)/rubicv-example
CARGO_BUILD=RUSTFLAGS="-C link-arg=-Tlink.x" cargo build --release --target $(TARGET)

# Automatically find all Rust source files
SOURCES=$(shell find src -name "*.rs")

.PHONY: all cargo rubicv print clean

# Default target
all: cargo rubicv

# riscv ELF
cargo: $(RUST_BINARY)

$(RUST_BINARY): $(SOURCES) Cargo.toml
	$(CARGO_BUILD)

# stripped .bin
rubicv: $(OUTPUT_BIN)

$(OUTPUT_BIN): $(RUST_BINARY)
	riscv64-unknown-elf-objcopy -O binary \
	    --strip-debug \
		--only-section=.text \
		$< $@

# Print the .bin file as a Rust byte array
print: $(OUTPUT_BIN)
	@echo -n "Bytes: [" && \
	hexdump -v -e '1/1 "0x%02X, "' $(OUTPUT_BIN) | sed 's/, $$//' && \
	echo "]"

# Clean target
clean:
	cargo clean
