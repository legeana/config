function pacu
    pac -U $argv
end

function pacn
    pacman $argv
end

function pacr
    pac -R $argv
end

function pacrs
    pacr -s $argv
end

function pacq
    pacn -Q $argv
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
    pacn -Ss $argv
end

function pacsuw
    pacsu --noconfirm -w $argv
end

function pacsuwy
    pacsuw -y $argv
end
