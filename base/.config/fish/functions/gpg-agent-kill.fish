function gpg-agent-kill --description 'Kill running gpg-agent and cleanup variables'
    gpgconf --kill gpg-agent
    set -e GPG_TTY
    set -e SSH_AUTH_SOCK
end
