function fish_greeting
    echo -s "shell: " (set_color green) "fish" (set_color normal)
    echo -s "id:  " (set_color magenta) (id) (set_color normal)
        echo -s "host:  " (set_color cyan) $hostname (set_color normal)
    echo -s "PWD: " (set_color normal) "$PWD" (set_color normal)
end
