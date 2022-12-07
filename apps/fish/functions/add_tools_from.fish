function add_tools_from
    set src $argv[1]
    set tools $argv[2..]
    if test -z "$src"
        echo "Must specify from directory as the first argument" >&2
        return 1
    end
    for tool in $tools
        alias "$tool=$src/$tool"
    end
end
