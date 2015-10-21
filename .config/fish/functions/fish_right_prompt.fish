function fish_right_prompt -d "Write out the right prompt"
    set -l external_status $status
    set -l external_cmd_duration $CMD_DURATION
    if [ (count $external_cmd_duration) -eq 0 ]
        set external_cmd_duration 0
    end
    switch "$external_status"
    case 0
        set_color green
    case '*'
        set_color red
    end
    echo -s -n $external_status ' '
    set_color magenta
    echo -s -n {$external_cmd_duration}ms ' '
    set_color cyan
    date "+%H:%M %a %d "
    set_color normal
end
