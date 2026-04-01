function uninstall-lazy
    rm -rf ~/.local/{state,share}/lazy-vim
    true > ~/.config/lazy-vim/lazy-lock.json  # Truncate.
    # Might be necessary to remove ~/.cache/nvim/luac as well.
end
