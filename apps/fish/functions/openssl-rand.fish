function openssl-rand -d 'openssl-rand num[K|M|G|T]'
    openssl rand -base64 $argv | tr -d '\n'
end
