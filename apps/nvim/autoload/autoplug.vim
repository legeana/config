let s:plug_url = 'https://raw.githubusercontent.com/junegunn/vim-plug/master/plug.vim'

let s:xdg_cache_home = empty($XDG_CACHE_HOME) ? $HOME . '/.cache' : $XDG_CACHE_HOME
let s:plug_dir = s:xdg_cache_home . '/vim-plug'
execute 'set runtimepath+=' . s:plug_dir

function! s:fetch(src, dst)
    silent exe '!curl -fL --create-dirs -o ' . a:dst . ' ' . a:src
endfunction

function! autoplug#begin()
    let g:autoplug_install = 0
    let autoload_plug_path = s:plug_dir . '/autoload/plug.vim'
    if !filereadable(autoload_plug_path)
        call s:fetch(s:plug_url, autoload_plug_path)
        execute 'source ' . fnameescape(autoload_plug_path)
        "autocmd VimEnter * PlugInstall --sync | source $MYVIMRC
        let g:autoplug_install = 1
    endif

    call plug#begin(s:plug_dir . '/plugins')
endfunction

function autoplug#end()
    call plug#end()
    if g:autoplug_install
        PlugInstall --sync
    endif
    unlet g:autoplug_install
endfunction
