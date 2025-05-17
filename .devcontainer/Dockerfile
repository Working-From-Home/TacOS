FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye
SHELL ["/bin/bash", "-euxo", "pipefail", "-c"]

RUN <<EOF
	apt-get update
	apt-get install -y --no-install-recommends \
		gcc-multilib \
		libc6-dev-i386 \
		qemu-system-i386 \
		grub-pc-bin \
		xorriso \
	;
	rm -rf /var/lib/apt/lists/*

	rustup target add i686-unknown-linux-gnu
	# rustup override set nightly-2025-05-10
EOF
