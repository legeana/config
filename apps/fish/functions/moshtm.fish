function moshtm -a ssh_host -a session -d 'Connect to a remote tmux session, usage: moshtm $ssh_host $session'
    mosh -- $ssh_host fish -c 'tm '(string escape $session)
end
