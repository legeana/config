function add_path_to_list
    if [ (count $argv) -gt 1 ]
        for arg in $argv[-1..2]
            if not contains $arg $$argv[1]; and [ -d $arg ]
                set $argv[1] $arg $$argv[1]
            end
        end
    end
end
