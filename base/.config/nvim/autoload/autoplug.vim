let s:plug_url = 'https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'

if has('nvim')
    let s:config_dir = stdpath('config')
else
    let s:config_dir = expand('~/.config/nvim')
endif

function! s:fetch(src, dst)
    silent exe '!curl -fL --create-dirs -o ' . a:dst . ' ' . a:src
endfunction

function! autoplug#begin(...)
    let g:autoplug_install = 0
    let autoload_plug_path = s:config_dir . '/autoload/plug.vim'
    if !filereadable(autoload_plug_path)
        call s:fetch(s:plug_url, autoload_plug_path)
        execute 'source ' . fnameescape(autoload_plug_path)
        echo fnameescape(autoload_plug_path)
        "autocmd VimEnter * PlugInstall --sync | source $MYVIMRC
        let g:autoplug_install = 1
    endif

    call call(function('plug#begin'), a:000)
endfunction

function autoplug#end()
    call plug#end()
    if g:autoplug_install
        PlugInstall --sync
    endif
    unlet g:autoplug_install
endfunction
