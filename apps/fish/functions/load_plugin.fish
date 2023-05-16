function load_plugin
    for plugin in $argv
        if test -d $plugin/functions
            add_to_function_path $plugin/functions
        end
        if test -d $plugin/completions
            add_to_complete_path $plugin/completions
        end
        if test -d $plugin/conf.d
            load_d $plugin/conf.d
        end
        if not test -d $plugin/functions
           and not test -d $plugin/conf.d
           and not test -d $plugin/config.d
            add_to_function_path $plugin
        end
    end
end
