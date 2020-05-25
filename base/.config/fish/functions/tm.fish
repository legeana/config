function tm -a session
    if test -z "$session"
        set session default
    end
    set env_overrides
    _tm_forward $session
    env $env_overrides tmux attach -t $session
    or env $env_overrides tmux new-session -s $session -c $HOME
end

complete --command=tm --no-files --arguments='(tmux list-sessions -F "#{session_name}")'

# The functions below operate under the assumption that tmux will not be used
# by more than one user simultaneously. The last connection wins.

set -g _TM_TMP "$HOME/.tmux-tmp"

function _tm_cleanup
    for f in "$_TM_TMP/"*
        if ! test -L $f
            continue
        end
        if ! test -e $f
            rm -f $f
        end
    end
end

function _tm_forward_socket --no-scope-shadowing -a session -a env -a value
    mkdir -p -m 700 "$_TM_TMP"
    set path "$_TM_TMP/$USER.$session.$env"
    ln -snf -- $$env $path
    set --append env_overrides "$env=$path"
end

function _tm_forward --no-scope-shadowing -a session
    _tm_cleanup
    # Restore FWD_SSH_AUTH_SOCK and forward it as SSH_AUTH_SOCK if possible.
    # FWD_SSH_AUTH_SOCK if set points to the original agent socket.
    set --append env_overrides --unset=FWD_SSH_AUTH_SOCK
    if ! set -q FWD_SSH_AUTH_SOCK || ! test -S $FWD_SSH_AUTH_SOCK
        _tm_forward_socket $session SSH_AUTH_SOCK $SSH_AUTH_SOCK
    else
        _tm_forward_socket $session SSH_AUTH_SOCK $FWD_SSH_AUTH_SOCK
    end
end
