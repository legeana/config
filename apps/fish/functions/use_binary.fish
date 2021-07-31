function use_binary
    set -l name (basename $argv[1])
    set -l bin $argv[1]
    alias $name $bin
end
