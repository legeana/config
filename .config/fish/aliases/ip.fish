function ip --wraps=ip
    command ip $fish_ip_args $argv
end

set -g fish_ip_args
if command ip -color -V >/dev/null ^&1
    set fish_ip_args $fish_ip_args -color
end
