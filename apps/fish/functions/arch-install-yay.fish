function arch-install-yay -d 'Install yay from AUR'
    set wd (mktemp -d)
    verbose-eval git clone https://aur.archlinux.org/yay.git $wd
    verbose-eval env --chdir=$wd makepkg --cleanbuild --syncdeps --install --force
    verbose-eval rm -rf $wd
end
