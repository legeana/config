function fish_prompt --description 'Write out the prompt'
    # Save status immediately to prevent unintended changes.
    set -l external_status $status

    # Just calculate this once, to save a few cycles when displaying the prompt
    if not set -q __fish_prompt_hostname
        set -g __fish_prompt_hostname (hostname | cut -d . -f 1)
    end

    set -l suffix
    set -l need_username
    set -l color_status
    switch $USER
    case root toor
        set suffix '#'
        set need_username ''
    case '*'
        set suffix '$'
        set need_username 1
    end

    switch "$external_status"
    case 0
        set color_status green
    case '*'
        set color_status red
    end

    if [ "$need_username" ]
        echo -n -s \
            (set_color magenta) "$USER" (set_color normal) \
            @
    end
    echo -n -s \
        (set_color cyan) "$__fish_prompt_hostname" (set_color normal) \
        ':' \
        (prompt_pwd) \
        (__fish_git_prompt) \
        (set_color $color_status) "$suffix " (set_color normal)
end
