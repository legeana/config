function add_to_path
    for dir in $argv[-1..1]
        if not contains $dir $PATH
            set PATH $dir $PATH
        end
    end
end
