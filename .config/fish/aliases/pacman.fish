if command --search pacman >/dev/null
    function pacorig
        if [ (id -u) = 0 ]
            pacman $argv
        else
            sudo pacman $argv
        end
    end
    if command --search pacaur >/dev/null
        function pac
            if [ (id -u) = 0 ]
                sudo -u $SUDO_USER pacaur $argv
            else
                pacaur $argv
            end
        end
        function pacuser
            pacaur $argv
        end
    else
        function pac
            if [ (id -u) = 0 ]
                pacman $argv
            else
                sudo pacman $argv
            end
        end
        function pacuser
            pacman $argv
        end
    end

    function pacu
        pac -U $argv
    end

    function pacr
        pac -R $argv
    end

    function pacrs
        pacr -s $argv
    end

    function pacq
        pacuser -Q $argv
    end

    function pacqo
        pacq -o $argv
    end

    function pacqs
        pacq -s $argv
    end

    function pacql
        pacq -l $argv
    end

    function pacqi
        pacq -i $argv
    end

    function pacs
        pac --needed -S $argv
    end

    function pacsu
        pacs -u $argv
    end

    function pacsy
        pacs -y $argv
    end

    function pacsuy
        pacsu -y $argv
    end

    function pacss
        pacuser -Ss $argv
    end

    function pacsuw
        pacorig --needed -Suw $argv
    end

    function pacsuwy
        pacsuw -y $argv
    end

    function packeys
        pacsy
        pacs archlinux-keyring
    end
end
