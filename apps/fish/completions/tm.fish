function __tm_tmux_sessions
    tmux list-sessions -F '#{session_name}'
end

function __tm_tmuxinator_projects
    if ! command -q tmuxinator
        return
    end
    tmuxinator completions start
end

function __complete_tm
    __tm_tmux_sessions | string replace -r '$' \t'tmux session'
    __tm_tmuxinator_projects | string replace -r '$' \t'tmuxinator project'
end

complete --command=tm --no-files --arguments='(__complete_tm)'
