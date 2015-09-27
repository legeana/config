function cdr
    set -l PPWD $PWD
    set -l CWD ..
    if set -q argv[1]
        set CWD $argv[1]
    end
    cd $CWD
    rmdir $PPWD ; or cd $PPWD
end
