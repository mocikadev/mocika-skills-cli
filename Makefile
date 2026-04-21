BINARY   := skm
VERSION  := $(shell grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
DIST_DIR := dist

TARGET_LINUX_X86_64  := x86_64-unknown-linux-musl
TARGET_LINUX_AARCH64 := aarch64-unknown-linux-musl
TARGET_MACOS_X86_64  := x86_64-apple-darwin
TARGET_MACOS_AARCH64 := aarch64-apple-darwin

CARGO  ?= cargo
PREFIX ?= $(HOME)/.local

ifeq ($(shell uname -s),Darwin)
  SHA256SUM := shasum -a 256
  HOST_OS   := darwin
else
  SHA256SUM := sha256sum
  HOST_OS   := linux
endif

.DEFAULT_GOAL := help

.PHONY: help build release test check fmt clippy clean install \
        build-linux-x86_64 build-linux-aarch64 \
        build-macos-x86_64 build-macos-aarch64 \
        dist dist-linux dist-macos

help:
	@printf "Usage: make <target> [CARGO=cargo] [PREFIX=~/.local]\n\n"
	@printf "Development\n"
	@printf "  build               Debug build (native)\n"
	@printf "  release             Release build (native)\n"
	@printf "  test                Run tests\n"
	@printf "  check               fmt --check + clippy\n"
	@printf "  fmt                 Format source code\n"
	@printf "  clippy              Run clippy lints\n"
	@printf "  clean               Remove build artifacts and dist/\n"
	@printf "  install             Install to PREFIX/bin  (default: ~/.local/bin)\n"
	@printf "\n"
	@printf "Cross-compilation (single target)\n"
	@printf "  build-linux-x86_64  Linux  x86_64  musl  [requires: musl-tools]\n"
	@printf "  build-linux-aarch64 Linux  aarch64 musl  [requires: aarch64-linux-musl-gcc]\n"
	@printf "  build-macos-x86_64  macOS  x86_64        [on macOS: native | on Linux: requires zig + cargo-zigbuild]\n"
	@printf "  build-macos-aarch64 macOS  aarch64        [on macOS: native | on Linux: requires zig + cargo-zigbuild]\n"
	@printf "\n"
	@printf "Distribution\n"
	@printf "  dist                Build all platform artifacts -> dist/\n"
	@printf "  dist-linux          Build Linux artifacts only\n"
	@printf "  dist-macos          Build macOS artifacts only\n"
	@printf "\n"
	@printf "Version: $(VERSION)\n"

build:
	$(CARGO) build

release:
	$(CARGO) build --release

test:
	$(CARGO) test

check: fmt-check clippy

fmt:
	$(CARGO) fmt

fmt-check:
	$(CARGO) fmt --check

clippy:
	$(CARGO) clippy -- -D warnings

clean:
	$(CARGO) clean
	rm -rf $(DIST_DIR)

install: release
	install -Dm755 target/release/$(BINARY) $(PREFIX)/bin/$(BINARY)
	@printf "installed $(PREFIX)/bin/$(BINARY)\n"

build-linux-x86_64:
	$(call ensure_target,$(TARGET_LINUX_X86_64))
	$(call require_linker,x86_64-linux-musl-gcc,apt install musl-tools)
	$(CARGO) build --release --target $(TARGET_LINUX_X86_64)

build-linux-aarch64:
	$(call ensure_target,$(TARGET_LINUX_AARCH64))
	$(call require_linker,aarch64-linux-musl-gcc,\
		Download a musl cross-toolchain from https://musl.cc\n\
		  or: cargo install cargo-zigbuild && make $$@ CARGO=cargo-zigbuild)
	$(CARGO) build --release --target $(TARGET_LINUX_AARCH64)

build-macos-x86_64:
	$(call ensure_target,$(TARGET_MACOS_X86_64))
	$(call require_macos_or_zigbuild)
	@if [ "$(HOST_OS)" = "darwin" ]; then \
		$(CARGO) build --release --target $(TARGET_MACOS_X86_64); \
	else \
		cargo zigbuild --release --target $(TARGET_MACOS_X86_64); \
	fi

build-macos-aarch64:
	$(call ensure_target,$(TARGET_MACOS_AARCH64))
	$(call require_macos_or_zigbuild)
	@if [ "$(HOST_OS)" = "darwin" ]; then \
		$(CARGO) build --release --target $(TARGET_MACOS_AARCH64); \
	else \
		cargo zigbuild --release --target $(TARGET_MACOS_AARCH64); \
	fi

dist-linux: build-linux-x86_64 build-linux-aarch64
	@mkdir -p $(DIST_DIR)
	cp target/$(TARGET_LINUX_X86_64)/release/$(BINARY)  $(DIST_DIR)/$(BINARY)-$(TARGET_LINUX_X86_64)
	cp target/$(TARGET_LINUX_AARCH64)/release/$(BINARY) $(DIST_DIR)/$(BINARY)-$(TARGET_LINUX_AARCH64)
	@printf "linux artifacts -> $(DIST_DIR)/\n"

dist-macos: build-macos-x86_64 build-macos-aarch64
	@mkdir -p $(DIST_DIR)
	cp target/$(TARGET_MACOS_X86_64)/release/$(BINARY)  $(DIST_DIR)/$(BINARY)-$(TARGET_MACOS_X86_64)
	cp target/$(TARGET_MACOS_AARCH64)/release/$(BINARY) $(DIST_DIR)/$(BINARY)-$(TARGET_MACOS_AARCH64)
	@printf "macos artifacts -> $(DIST_DIR)/\n"

dist: dist-linux dist-macos
	cd $(DIST_DIR) && $(SHA256SUM) $(BINARY)-* > SHA256SUMS.txt
	@printf "\ndist/\n"
	@ls -lh $(DIST_DIR)

# ── Helpers ───────────────────────────────────────────────────────────────────

define ensure_target
	@rustup target list --installed 2>/dev/null | grep -qx "$(1)" || { \
		printf "rustup target '$(1)' not installed -- running: rustup target add $(1)\n"; \
		rustup target add $(1); \
	}
endef

define require_linker
	@command -v $(1) >/dev/null 2>&1 || { \
		printf "error: linker '$(1)' not found\n"; \
		printf "  Install: $(2)\n"; \
		exit 1; \
	}
endef

define require_macos_or_zigbuild
	@if [ "$(HOST_OS)" = "darwin" ]; then \
		true; \
	elif command -v cargo-zigbuild >/dev/null 2>&1 && command -v zig >/dev/null 2>&1; then \
		true; \
	else \
		printf "error: macOS targets on Linux require zig + cargo-zigbuild\n"; \
		printf "  1. Install zig:            https://ziglang.org/download/\n"; \
		printf "  2. Install cargo-zigbuild: cargo install cargo-zigbuild\n"; \
		exit 1; \
	fi
endef
