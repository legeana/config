function sshtm -d 'Connect to a remote tmux session, usage: sshtm $sshargs $session'
    set sshargs $argv[1..-2]
    set session $argv[-1]
    if ! count $sshargs >/dev/null
        echo 'Must specify $sshargs' >&2
        return 1
    end
    ssh -t $sshargs -- fish -c (string escape 'tm '(string escape $session))
end
