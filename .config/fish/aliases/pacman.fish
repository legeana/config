if command --search pacman >/dev/null
    if [ (id -u) = 0 ]
        alias pacorig='pacman'
    else
        alias pacorig='sudo pacman'
    end
    if command --search pacaur >/dev/null
        if [ (id -u) = 0 ]
            alias pac='sudo -u '$SUDO_USER' pacaur'
        else
            alias pac='pacaur'
        end
        alias pacuser='pacaur'
    else
        if [ (id -u) = 0 ]
            alias pac='pacman'
        else
            alias pac='sudo pacman'
        end
        alias pacuser='pacman'
    end

    alias pacu='pac -U'
    alias pacr='pac -R'
    alias pacrs='pacr -s'
    alias pacq='pacuser -Q'
    alias pacqo='pacq -o'
    alias pacqs='pacq -s'
    alias pacql='pacq -l'
    alias pacqi='pacq -i'
    alias pacs='pac --needed -S'
    alias pacsu='pacs -u'
    alias pacsy='pacs -y'
    alias pacsuy='pacsu -y'
    alias pacss='pacuser -Ss'
    alias pacsc='pacs -c'
    alias pacsuw='pacorig --needed -Suw'
    alias pacsuwy='pacsuw -y'
    alias pack='pacman-key'

    function packeys
        pacsy
        pacs archlinux-keyring
    end
end
