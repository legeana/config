function load_f
    for file in $argv
        if [ -f $file ]
            source $file
        end
    end
end
