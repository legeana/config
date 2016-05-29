begin
    set -l grc_commands \
        last \
        netstat \
        ping \
        traceroute

    if which grc >/dev/null ^&1
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
