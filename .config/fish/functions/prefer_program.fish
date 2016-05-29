function prefer_program
    for program in $argv[2..-1]
        if which $program >/dev/null ^&1
            set -gx $argv[1] $program
            break
        end
    end
end
