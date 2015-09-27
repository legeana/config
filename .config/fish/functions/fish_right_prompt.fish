function fish_right_prompt -d "Write out the right prompt"
    switch "$status"
    case 0
        set_color green
    case '*'
        set_color red
    end
    echo -s -n $status ' '
    set_color cyan
    date "+%H:%M %a %d "
    set_color normal
end

#export RPROMPT='%(?.%F{green}.%F{red})%?%f %F{cyan}%U%T%u %U%w%u%f'
