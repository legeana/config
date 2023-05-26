function fisher_install_needed
    set -l installed (fisher list)
    set -l needed
    for i in $argv
        if ! contains $i $installed
            set -a needed $i
        end
    end
    if count $needed >/dev/null
        fisher install $needed
    end
end
