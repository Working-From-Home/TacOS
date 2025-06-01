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
			./configure --prefix="${HOME}/.kfs/.mtools"
			make -j$(nproc)
			make install
		)
		rm -rf "/tmp/mtools-${mtools_version}" "/tmp/mtools.tar.gz"
	fi
	export PATH="${HOME}/.kfs/.mtools/bin:$PATH"
	grep -qxF 'export PATH="${HOME}/.kfs/.mtools/bin:$PATH"' "${HOME}/.zshrc" || echo 'export PATH="${HOME}/.kfs/.mtools/bin:$PATH"' >> "${HOME}/.zshrc"
	grep -qxF 'export PATH="${HOME}/.kfs/.mtools/bin:$PATH"' "${HOME}/.bashrc" || echo 'export PATH="${HOME}/.kfs/.mtools/bin:$PATH"' >> "${HOME}/.bashrc"
}

install_m4(){
	local m4_version="1.4.20"
	if ! command -v m4 &> /dev/null || [[ $(m4 --version | head -n1 | grep -oE '[0-9]+\\.[0-9]+\\.[0-9]+') != "$m4_version" ]]; then
		echo "m4 $m4_version is not installed. Fetching sources and installing..."
		wget -qO /tmp/m4.tar.gz "https://ftp.gnu.org/gnu/m4/m4-${m4_version}.tar.gz"
		(
			cd /tmp
			tar xzf "/tmp/m4.tar.gz"
			cd "m4-${m4_version}"
			./configure --prefix="${HOME}/.kfs/.m4"
			make -j$(nproc)
			make install
		)
		rm -rf "/tmp/m4-${m4_version}" "/tmp/m4.tar.gz"
	fi
	export PATH="${HOME}/.kfs/.m4/bin:$PATH"
	grep -qxF 'export PATH="${HOME}/.kfs/.m4/bin:$PATH"' "${HOME}/.zshrc" || echo 'export PATH="${HOME}/.kfs/.m4/bin:$PATH"' >> "${HOME}/.zshrc"
	grep -qxF 'export PATH="${HOME}/.kfs/.m4/bin:$PATH"' "${HOME}/.bashrc" || echo 'export PATH="${HOME}/.kfs/.m4/bin:$PATH"' >> "${HOME}/.bashrc"
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
			./configure --prefix="${HOME}/.kfs/.bison"
			make -j$(nproc)
			make install
		)
		rm -rf "/tmp/bison-${bison_version}" "/tmp/bison.tar.gz"
	fi
	export PATH="${HOME}/.kfs/.bison/bin:$PATH"
	grep -qxF 'export PATH="${HOME}/.kfs/.bison/bin:$PATH"' "${HOME}/.zshrc" || echo 'export PATH="${HOME}/.kfs/.bison/bin:$PATH"' >> "${HOME}/.zshrc"
	grep -qxF 'export PATH="${HOME}/.kfs/.bison/bin:$PATH"' "${HOME}/.bashrc" || echo 'export PATH="${HOME}/.kfs/.bison/bin:$PATH"' >> "${HOME}/.bashrc"
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
			./configure --prefix="${HOME}/.kfs/.flex"
			make -j$(nproc)
			make install
		)
		rm -rf "/tmp/flex-${flex_version}" "/tmp/flex.tar.gz"
	fi
	export PATH="${HOME}/.kfs/.flex/bin:$PATH"
	grep -qxF 'export PATH="${HOME}/.kfs/.flex/bin:$PATH"' "${HOME}/.zshrc" || echo 'export PATH="${HOME}/.kfs/.flex/bin:$PATH"' >> "${HOME}/.zshrc"
	grep -qxF 'export PATH="${HOME}/.kfs/.flex/bin:$PATH"' "${HOME}/.bashrc" || echo 'export PATH="${HOME}/.kfs/.flex/bin:$PATH"' >> "${HOME}/.bashrc"
}

install_autoconf() {
	local autoconf_version="2.71"
	if ! command -v autoconf &> /dev/null; then
		echo "autoconf is not installed. Fetching sources and installing..."
		wget -qO /tmp/autoconf.tar.gz "https://ftp.gnu.org/gnu/autoconf/autoconf-${autoconf_version}.tar.gz"
		(
			cd /tmp
			tar xzf "/tmp/autoconf.tar.gz"
			cd "autoconf-${autoconf_version}"
			./configure --prefix="${HOME}/.kfs/.autoconf"
			make -j$(nproc)
			make all install
		)
		rm -rf "/tmp/autoconf-${autoconf_version}" "/tmp/autoconf.tar.gz"
	fi
	export PATH="${HOME}/.kfs/.autoconf/bin:$PATH"
	grep -qxF 'export PATH="${HOME}/.kfs/.autoconf/bin:$PATH"' "${HOME}/.zshrc" || echo 'export PATH="${HOME}/.kfs/.autoconf/bin:$PATH"' >> "${HOME}/.zshrc"
	grep -qxF 'export PATH="${HOME}/.kfs/.autoconf/bin:$PATH"' "${HOME}/.bashrc" || echo 'export PATH="${HOME}/.kfs/.autoconf/bin:$PATH"' >> "${HOME}/.bashrc"
}

install_automake() {
	local automake_version="1.15"
	local automake_url="https://ftp.gnu.org/gnu/automake/automake-${automake_version}.tar.gz"
	if ! command -v automake-1.15 &> /dev/null; then	
		echo "automake-1.15 is not installed. Downloading and building..."
		wget -qO /tmp/automake.tar.gz "$automake_url"
		(
			cd /tmp
			tar xzf automake.tar.gz
			cd "automake-${automake_version}"
			./configure --prefix="${HOME}/.kfs/.automake-1.15"
			make -j$(nproc)
			make install
		)
		"${HOME}/.kfs/.automake-1.15/bin/automake-1.15" --add-missing
		rm -rf /tmp/automake.tar.gz /tmp/automake-${automake_version}
	fi
	grep -qxF 'export PATH="${HOME}/.kfs/.automake-1.15/bin:$PATH"' ${HOME}/.zshrc || echo 'export PATH="${HOME}/.kfs/.automake-1.15/bin:$PATH"' >> ${HOME}/.zshrc
	grep -qxF 'export PATH="${HOME}/.kfs/.automake-1.15/bin:$PATH"' ${HOME}/.bashrc || echo 'export PATH="${HOME}/.kfs/.automake-1.15/bin:$PATH"' >> ${HOME}/.bashrc
}

install_gettext() {
	local gettext_version="0.25"
	if ! command -v autopoint &> /dev/null; then
		echo "gettext is not installed. Fetching sources and installing..."
		wget -qO /tmp/gettext.tar.gz "https://ftp.gnu.org/gnu/gettext/gettext-${gettext_version}.tar.gz"
		(
			cd /tmp
			tar xzf /tmp/gettext.tar.gz
			cd "gettext-${gettext_version}"
			./configure --prefix=${HOME}/.kfs/.gettext
			make -j$(nproc)
			make install
		)
		rm -rf "/tmp/gettext-${gettext_version}" "/tmp/gettext.tar.gz"
	fi
	export PATH="${HOME}/.kfs/.gettext/bin:$PATH"
	grep -qxF 'export PATH="${HOME}/.kfs/.gettext/bin:$PATH"' ${HOME}/.zshrc || echo 'export PATH="${HOME}/.kfs/.gettext/bin:$PATH"' >> ${HOME}/.zshrc
	grep -qxF 'export PATH="${HOME}/.kfs/.gettext/bin:$PATH"' ${HOME}/.bashrc || echo 'export PATH="${HOME}/.kfs/.gettext/bin:$PATH"' >> ${HOME}/.bashrc
}

install_grub_pc() {
	local grub_version="2.12"
	if ! command -v grub-pc &> /dev/null; then
		echo "GRUB (grub-pc) is not installed. Fetching sources and installing..."
		wget -qO /tmp/grub.tar.xz "https://ftp.gnu.org/gnu/grub/grub-${grub_version}.tar.xz"
		install_gettext
		install_bison
		install_flex
		install_m4
		install_autoconf
		install_automake
		export ACLOCAL_PATH="${HOME}/.kfs/.automake-1.15/share/aclocal:${HOME}/.kfs/.gettext/share/aclocal:/usr/share/aclocal"
		export M4PATH="${HOME}/.kfs/.autoconf/share/autoconf:${HOME}/.kfs/.automake-1.15/share/aclocal:${HOME}/.kfs/.gettext/share/aclocal:/usr/share/aclocal"
		(
			cd /tmp
			tar xf "/tmp/grub.tar.xz"
			cd "grub-${grub_version}"
			echo depends bli part_gpt > grub-core/extra_deps.lst
			./autogen.sh
			./configure --prefix=${HOME}/.kfs/.grub --with-platform=pc --disable-werror --disable-dependency-tracking --disable-efiemu # https://drlm-docs.readthedocs.io/en/latest/building_grub2.html
			# ./configure --prefix=${HOME}/.kfs/.grub --with-platform=pc #--disable-werror
			unset {C,CPP,CXX,LD}FLAGS
			make
			make install
		)
		rm -rf "/tmp/grub-${grub_version}" "/tmp/grub.tar.xz"
	fi
	export PATH="${HOME}/.kfs/.grub/bin:$PATH"
	grep -qxF 'export PATH="${HOME}/.kfs/.grub/bin:$PATH"' ${HOME}/.zshrc || echo 'export PATH="${HOME}/.kfs/.grub/bin:$PATH"' >> ${HOME}/.zshrc
	grep -qxF 'export PATH="${HOME}/.kfs/.grub/bin:$PATH"' ${HOME}/.bashrc || echo 'export PATH="${HOME}/.kfs/.grub/bin:$PATH"' >> ${HOME}/.bashrc
}

clean_installed_soft() {
	echo "Cleaning all locally installed tools..."
	rm -rf ${HOME}/.kfs
	sed -i '/export PATH=\\"\\$HOME\\/.kfs\//d' ${HOME}/.zshrc ${HOME}/.bashrc || true
	echo "Done. Please restart your shell or source your rc file."
}

main(){
	if [[ "${1:-}" == "--clean" ]]; then
		clean_installed_soft
		exit 0
	fi
	echo "Setting up 42 Linux environment..."
	install_rustup
	install_mtools
	export PATH="${HOME}/.kfs/bin:$PATH"
	grep -qxF 'export PATH="${HOME}/.kfs/bin:$PATH"' "${HOME}/.zshrc" || echo 'export PATH="${HOME}/.kfs/bin:$PATH"' >> "${HOME}/.zshrc"
	grep -qxF 'export PATH="${HOME}/.kfs/bin:$PATH"' "${HOME}/.bashrc" || echo 'export PATH="${HOME}/.kfs/bin:$PATH"' >> "${HOME}/.bashrc"
	install_grub_pc
	rustup component add rust-src
}

main "$@"

# TODO if error happen
#     mawk: ./genmoddep.awk: line 110: function asorti never defined
#     make[3]: *** [Makefile:49030: moddep.lst] Error 1
# Please make sure gawk is available and used by default for awk.
#  https://lists.buildroot.org/pipermail/buildroot/2023-December/366838.html