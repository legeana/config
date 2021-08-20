function prefer_program
    for program in $argv[2..-1]
        if command -q $program
            set -gx $argv[1] $program
            break
        end
    end
end
