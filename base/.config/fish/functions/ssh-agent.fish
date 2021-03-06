function ssh-agent --description 'launch the ssh-agent and add the id_rsa identity'
    if begin
            set -q SSH_AGENT_PID
            and kill -0 $SSH_AGENT_PID
            and ps -p $SSH_AGENT_PID | grep -q '[s]sh-agent'
    end
        echo "ssh-agent running on pid $SSH_AGENT_PID"
    else
        set -e SSH_AUTH_SOCK
        set -e SSH_AGENT_PID
        eval (command ssh-agent -c | sed 's/^setenv/set -Ux/')
    end
    ssh-add-if-necessary
end
