if command --search pacman >/dev/null
    if [ "$(id -u)" = 0 ]
        alias pacorig='pacman'
        alias pack='pacman-key'
        alias pac='pacman'
        alias pacuser='pacman'
    else
        alias pacorig='sudo pacman'
        alias pack='sudo pacman-key'
        alias pac='sudo pacman'
        alias pacuser='pacman'
    end

    if command --search yay >/dev/null
        if [ "$(id -u)" = 0 ]
            # can use yay as root
            if set -q SUDO_USER
                alias pac="sudo -u $SUDO_USER yay --sudoloop"
                alias pacuser='yay --sudoloop'
            end
        else
            alias pac='yay --sudoloop'
            alias pacuser='yay --sudoloop'
        end
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
    alias pacsuw='pacorig --needed --noconfirm -Suw'
    alias pacsuwy='pacsuw -y'

    function packeys
        pacsy
        pacs archlinux-keyring
    end
end
