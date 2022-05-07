function svim -w vim
    if command -q nvim
        sudo -E nvim $argv
    else
        sudo -E vim $argv
    end
end
