function fish_prompt --description 'Write out the prompt'
    # Save these values at the very beginning of the prompt.
    # Do not attempt to measure performance of this code since it's trivial
    # and measurements will change $status and $CMD_DURATION.
    set -g __fish_prompt_status $status
    set -g __fish_prompt_cmd_duration $CMD_DURATION
    if [ (count $__fish_prompt_cmd_duration) -eq 0 ]
        set __fish_prompt_cmd_duration 0
    end

    __fish_prompt_profile begin

    __fish_prompt_signal
    __fish_prompt_profile signal

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

    __fish_prompt_profile pre
    __fish_prompt_info
    __fish_prompt_profile info
    __fish_prompt_context  # TODO optimize git prompt
    __fish_prompt_profile context
    __fish_prompt_input
end

set -g __fish_prompt_signals (string split ' ' (kill -l))
function __fish_prompt_signal
    set signum 0
    set -g __fish_prompt_signame ""
    if [ $__fish_prompt_status -gt 128 ]
        set signum (math $__fish_prompt_status - 128)
        if [ 1 -le $signum ] && [ $signum -le (count $__fish_prompt_signals) ]
            set -g __fish_prompt_signame $__fish_prompt_signals[$signum]
        end
    end
end

function __fish_prompt_profile
    if set --query __fish_prompt_profile_enabled
        printf '%10s %s\n' "<$argv[1]>" (date '+%s.%N')
    end
end

function __fish_prompt_cmd_saver --on-event fish_postexec
    if string length --quiet $argv[1]
        set -l arg $argv[1]
        # Use quoted substitution since commands may return multiple tokens.
        set -l arg (string replace \n ' ' "$arg")
        set -l arg (string replace -r '^\s*(\S+)\s.*$' '$1' "$arg")
        set -g __fish_prompt_cmd (string replace -r '^([^ ]*/)?([^/ ]+)(\s.*)?$' '$2' $arg)
    end
end

function __fish_prompt_sep
    if test $COLUMNS -gt $argv[1]
        echo -n ' '
    else
        echo
    end
end

function __fish_prompt_info
    set -g __fish_prompt_info_threshold 50
    if ! set -q fish_prompt_info_modules
        set -g fish_prompt_info_modules hostinfo date
    end
    for module in $fish_prompt_info_modules
        if eval "__fish_prompt_$module"
            __fish_prompt_sep $__fish_prompt_info_threshold
        end
    end
    __fish_prompt_result
    set -e __fish_prompt_info_threshold
    echo
end

function __fish_prompt_date
    set_color -o brblue
    if test $COLUMNS -gt $__fish_prompt_info_threshold
        echo -n '['
    end
    echo -s -n (date "+%H:%M %a %d")
    if test $COLUMNS -gt $__fish_prompt_info_threshold
        echo -n ']'
    end
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
    echo -n $(string replace -r '\..*$' '' $hostname)
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

function __filter_color_codes
    string replace --regex '\x1B\[[0-9;]*[JKmsu]' '' $argv
end

function __fish_prompt_context
    set -l indent (set_color red)'>'(set_color yellow)'>'(set_color green)'> '(set_color normal)
    set -l pwd (__fish_prompt_pwd)
    set -l git (__fish_prompt_git)
    set -l length (string length (__filter_color_codes "$indent$pwd$git"))
    echo -n $indent
    if test $COLUMNS -lt $length
        echo -n (prompt_pwd)
    else
        echo -n $pwd
    end
    echo $git
end

function __fish_prompt_git
    if is_local_filesystem $PWD
        __fish_git_prompt
    end
end

function __fish_prompt_input
    echo -n -s (set_color $__fish_prompt_color) "$__fish_prompt_suffix " (set_color normal)
end
