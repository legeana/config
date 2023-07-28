if command -q manpath
    set -e MANPATH
    set -gx MANPATH string replace --all : \n (manpath)
end
