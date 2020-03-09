for s:third_party in glob('~/.config/nvim/third_party/**/*.vim', 0, 1)
    execute 'source ' . fnameescape(s:third_party)
endfor
for s:third_party in glob('~/.config/nvim/after/third_party/**/*.vim', 0, 1)
    execute 'source ' . fnameescape(s:third_party)
endfor
