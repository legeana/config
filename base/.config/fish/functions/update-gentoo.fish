function update-gentoo
    set -l cmd 'emerge -DuN world && emerge --depclean && revdep-rebuild && prelink -amR'
    if [ (id -u) = 0 ]
        eval $cmd
    else
        echo 'You are not root!'
        echo 'Trying to use sudo'
        sudo $SHELL -c $cmd
    end
end
