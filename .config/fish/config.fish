source ~/.fishlocalrc

for file in ~/.config/fish/aliases/*.fish
    source $file
end

# GRC
begin
    set -l grc_commands \
        last \
        netstat \
        ping \
        traceroute

    if which grc >/dev/null
        function grc
            command grc --colour=auto $argv
        end
        for cmd in $grc_commands
            function $cmd
                command grc $cmd $argv
            end
        end
    end
end

if which vim >/dev/null
    function vi
        vim $argv
    end
end

if [ -n "$DISPLAY" ]
    set -g MPLAYER_PROFILE x
else if [ -n "$TMUX" -o -n "$SSH_CLIENT" -o "$TERM" = screen ]
    set -g MPLAYER_PROFILE audio
else
    set -g MPLAYER_PROFILE console
end
function mplayer
    command mplayer -profile "$MPLAYER_PROFILE" $argv
end
