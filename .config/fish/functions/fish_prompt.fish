function fish_prompt --description 'Write out the prompt'
    set -g __fish_prompt_status $status
    set -g __fish_prompt_cmd_duration $CMD_DURATION
    if [ (count $__fish_prompt_cmd_duration) -eq 0 ]
        set __fish_prompt_cmd_duration 0
    end
    set -g __fish_prompt_cmd (string replace -r '^(\w+)(\s.*)?$' '$1' $history[1])

    # Just calculate this once, to save a few cycles when displaying the prompt
    if not set -q __fish_prompt_hostname
        set -g __fish_prompt_hostname (hostname | cut -d . -f 1)
    end

    set -g __fish_prompt_signal 0
    set -g __fish_prompt_signame ""
    if [ $__fish_prompt_status -gt 128 ]
        set -g __fish_prompt_signal (math "$__fish_prompt_status-128")
        if [ $__fish_prompt_signal -le 64 ]
            set -g __fish_prompt_signame (kill --list=$__fish_prompt_signal)
        end
    end

    switch $USER
    case root toor
        set -g __fish_prompt_suffix '#'
    case '*'
        set -g __fish_prompt_suffix '$'
    end

    switch $__fish_prompt_status
    case 0
        set -g __fish_prompt_color green
    case '*'
        set -g __fish_prompt_color red
    end

    __fish_prompt_info
    __fish_prompt_context
    __fish_prompt_input
end

function __fish_prompt_info
    __fish_prompt_hostinfo
    echo -n ' '
    __fish_prompt_date
    echo -n ' '
    __fish_prompt_result
    echo
end

function __fish_prompt_date
    set_color -o brblue
    echo -n '['
    echo -s -n (date "+%H:%M %a %d")
    echo -n ']'
    set_color normal
end

function __fish_prompt_hostinfo
    if [ $__fish_prompt_suffix = '$' ]
        set_color magenta
        echo -n $USER
        set_color normal
        echo -n @
    end
    set_color cyan
    echo -n $__fish_prompt_hostname
    set_color normal
end

function __fish_prompt_result
    set_color magenta
    echo -s -n {$__fish_prompt_cmd_duration}ms ' '
    set_color $__fish_prompt_color
    echo -n '<'
    echo -n $__fish_prompt_status
    if [ $__fish_prompt_signame ]
        echo -s -n ' ' $__fish_prompt_signame
    end
    echo -n '>'
    set_color normal
    echo -n ' '
    echo -n $__fish_prompt_cmd
end

function __fish_prompt_pwd
    echo -n (string replace -r '^'"$HOME"'($|/)' '~$1' $PWD)
end

function __fish_prompt_context
    echo -s -n (set_color red)'❯'(set_color yellow)'❯'(set_color green)'❯ '(set_color normal)
    #echo -s -n $PWD
    __fish_prompt_pwd
    __fish_git_prompt
    echo
end

function __fish_prompt_input
    echo -n -s (set_color $__fish_prompt_color) "$__fish_prompt_suffix " (set_color normal)
end
