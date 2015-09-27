function load_d
    for file in $argv[1]/*.fish
        source $file
    end
end
