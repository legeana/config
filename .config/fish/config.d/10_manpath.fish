manpath | sed -r 's|:| |g;s|^|set -gx MANPATH |' | source
