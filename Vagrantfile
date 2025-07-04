
Vagrant.configure("2") do |config|
  config.vm.box = "ubuntu/focal64"
  config.vagrant.plugins = "vagrant-reload"

  config.vm.provider "virtualbox" do |vb|    
    vb.cpus = 8
    vb.memory = 8192
    vb.gui = true
    vb.customize ['modifyvm', :id, '--clipboard-mode', 'bidirectional']
    vb.customize ['modifyvm', :id, '--draganddrop', 'bidirectional']
    vb.customize ["modifyvm", :id, "--vram", "128"]
    vb.customize ["modifyvm", :id, "--graphicscontroller", "vmsvga"]
    vb.customize ["modifyvm", :id, "--nested-hw-virt", "on"]
  end

  config.vm.synced_folder ".", "/home/vagrant/Desktop/tacos", type: "virtualbox"
  config.vm.provision "shell", name: "Setting up Vm", privileged: false,  inline: <<-SHELL
    set -eux

    sudo apt-get update
    DEBIAN_FRONTEND=noninteractive sudo apt-get install -y --no-install-recommends \
      gdm3 \
      ubuntu-desktop-minimal \
      gcc-multilib \
      make \
      libc6-dev-i386 \
      qemu-system-i386 \
      grub-pc-bin \
      xorriso \
      zsh \
    ;

    echo "Installing rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

    curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh -o /tmp/install.sh
    sed -i 's/CHSH=no/CHSH=yes/g' /tmp/install.sh 
    echo "Y" | sh /tmp/install.sh
    rm /tmp/install.sh

    echo "display manager and desktop installed."

    # curl -sfL https://get.docker.com | sudo sh -
    # sudo usermod -aG docker "$USER"
    # echo "docker installed."

    sudo snap install --classic code
    echo "vscode installed."

    sudo snap install firefox
    echo "firefox installed."
  SHELL
  
  config.vm.provision :reload

  config.vm.provision "shell", name: "Finishing setup...", privileged: false, inline: <<-SHELL
    set -ex
  
    gsettings set org.gnome.shell favorite-apps "$(gsettings get org.gnome.shell favorite-apps | sed s/.$//), 'org.gnome.Terminal.desktop', 'code_code.desktop', 'firefox_firefox.desktop', 'virtualbox.desktop']"
    rustup target add i686-unknown-linux-gnu


  SHELL

end