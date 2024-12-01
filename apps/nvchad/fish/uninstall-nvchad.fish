function uninstall-nvchad
    rm -rf ~/.local/{state,share}/nvchad
    true > ~/.config/nvchad/lazy-lock.json  # Truncate.
    # Might be necessary to remove ~/.cache/nvim/luac as well.
end
