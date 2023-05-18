if command -q batcat
    alias cat='batcat -p'
    alias bat=batcat
else if command -q bat
    alias cat='bat -p'
end
