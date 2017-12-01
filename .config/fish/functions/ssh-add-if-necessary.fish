function ssh-add-if-necessary --description 'Add ssh identity only if it is not loaded'
    if count $argv >/dev/null
        for identity in $argv
            set -l fingerprint (ssh-keygen -lf $identity | awk '{print $2}')
            ssh-add -l | grep -q "$fingerprint"
                or ssh-add $identity
        end
    else
        ssh-add-if-necessary $HOME/.ssh/id_rsa
    end
end
