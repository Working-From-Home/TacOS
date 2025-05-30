#!/bin/bash

set -eux

install_rustup(){
	if ! command -v rustup &> /dev/null; then
		echo "rustup is not installed. Installing..."
		curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
			| RUSTUP_INIT_SKIP_PATH_CHECK=yes sh
	 	source $HOME/.cargo/env
	fi
}
install_mtools(){
	local mtools_version="4.0.48"
	if ! command -v mtools &> /dev/null; then
		echo "mtools is not installed. Fetching sources and installing..."
		wget -qO /tmp/mtools.tar.gz "https://ftp.gnu.org/gnu/mtools/mtools-${mtools_version}.tar.gz"
		(
			cd /tmp
			tar xzf "/tmp/mtools.tar.gz"
			cd "mtools-${mtools_version}"
			./configure --prefix=$HOME/.mtools
			make
			make install
			rm -rf "/tmp/mtools-${mtools_version}" "/tmp/mtools.tar.gz"
		)
	fi
	export PATH="$HOME/.mtools/bin:$PATH"
	grep -qxF 'export PATH="$HOME/.mtools/bin:$PATH"' $HOME/.zshrc || echo 'export PATH="$HOME/.mtools/bin:$PATH"' >> $HOME/.zshrc
	grep -qxF 'export PATH="$HOME/.mtools/bin:$PATH"' $HOME/.bashrc || echo 'export PATH="$HOME/.mtools/bin:$PATH"' >> $HOME/.bashrc
}

install_bison(){
	local bison_version="3.8.2"
	if ! command -v bison &> /dev/null; then
		echo "bison is not installed. Fetching sources and installing..."
		wget -qO /tmp/bison.tar.gz "https://ftp.gnu.org/gnu/bison/bison-${bison_version}.tar.gz"
		(
			cd /tmp
			tar xzf "/tmp/bison.tar.gz"
			cd "bison-${bison_version}"
			./configure --prefix=$HOME/.bison
			make
			make install
			rm -rf "/tmp/bison-${bison_version}" "/tmp/bison.tar.gz"
		)
	fi
	export PATH="$HOME/.bison/bin:$PATH"
	# And ensure future shells have it
	grep -qxF 'export PATH="$HOME/.bison/bin:$PATH"' $HOME/.zshrc || echo 'export PATH="$HOME/.bison/bin:$PATH"' >> $HOME/.zshrc
	grep -qxF 'export PATH="$HOME/.bison/bin:$PATH"' $HOME/.bashrc || echo 'export PATH="$HOME/.bison/bin:$PATH"' >> $HOME/.bashrc
}

install_flex(){
	local flex_version="2.6.4"
	if ! command -v flex &> /dev/null; then
		echo "flex is not installed. Fetching sources and installing..."
		wget -qO /tmp/flex.tar.gz "https://github.com/westes/flex/releases/download/v${flex_version}/flex-${flex_version}.tar.gz"
		(
			cd /tmp
			tar xzf "/tmp/flex.tar.gz"
			cd "flex-${flex_version}"
			./configure --prefix=$HOME/.flex
			make
			make install
			rm -rf "/tmp/flex-${flex_version}" "/tmp/flex.tar.gz"
		)
	fi
	export PATH="$HOME/.flex/bin:$PATH"
	grep -qxF 'export PATH="$HOME/.flex/bin:$PATH"' $HOME/.zshrc || echo 'export PATH="$HOME/.flex/bin:$PATH"' >> $HOME/.zshrc
	grep -qxF 'export PATH="$HOME/.flex/bin:$PATH"' $HOME/.bashrc || echo 'export PATH="$HOME/.flex/bin:$PATH"' >> $HOME/.bashrc
}

install_grub_pc_bin() {
	local grub_version="2.12"
	if ! command -v grub-pc &> /dev/null; then
		echo "GRUB (grub-pc) is not installed. Fetching sources and installing..."
		wget -qO /tmp/grub.tar.xz "https://ftp.gnu.org/gnu/grub/grub-${grub_version}.tar.xz"
		install_bison
		install_flex
		(
			cd /tmp
			tar xf "/tmp/grub.tar.xz"
			cd "grub-${grub_version}"
			./configure --prefix=$HOME/.grub --with-platform=pc #--disable-werror #
			make -j$(nproc)
			make install
			rm -rf "/tmp/grub-${grub_version}" "/tmp/grub.tar.xz"
		)
	fi
	export PATH="$HOME/.grub/bin:$PATH"
	grep -qxF 'export PATH="$HOME/.grub/bin:$PATH"' $HOME/.zshrc || echo 'export PATH="$HOME/.grub/bin:$PATH"' >> $HOME/.zshrc
	grep -qxF 'export PATH="$HOME/.grub/bin:$PATH"' $HOME/.bashrc || echo 'export PATH="$HOME/.grub/bin:$PATH"' >> $HOME/.bashrc
}

main(){
	echo "Setting up 42 Linux environment..."
	install_rustup
	install_mtools
	install_grub_pc_bin
	rustup component add rust-src
}

main "$@"
