function load_plugins
    for plugin_dir in $argv
        for plugin in $plugin_dir/*
            add_plugin $plugin
        end
    end
end
