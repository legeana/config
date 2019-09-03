function load_d
    for dir in $argv
        for file in $dir/*.fish
            source $file
        end
    end
end
