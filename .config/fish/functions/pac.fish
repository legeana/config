function pac
    if [ (id -u) = 0 ]
        pacman $argv
    else
        sudo pacman $argv
    end
end
