function source_d -d 'Sources all *.fish files in given directories'
    for dir in $argv
        for file in $dir/*.fish
            source $file
        end
    end
end
