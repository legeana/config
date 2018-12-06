function dirf
    if count $argv >/dev/null
        set arg $argv[1]
    else
        set arg .
    end
    find $arg -type d | sed -e "s/[^-][^\/]*\//  |/g" -e "s/|\([^ ]\)/|-\1/"
end
