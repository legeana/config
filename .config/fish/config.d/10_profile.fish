env -i HOME=$HOME sh -l -c 'source /etc/profile ; printenv' | \
    sed -e '/^PWD=/d;/^SHLVL=/d;/^_/d;/PATH/s/:/ /g;s/=/ /;s/^/set -x /' | \
    source

manpath | sed 's|:| |g;s|^|set -gx MANPATH |' | source
