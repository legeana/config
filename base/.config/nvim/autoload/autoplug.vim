function! autoplug#begin(...)
    let g:autoplug_install = 0
    let autoload_plug_path = stdpath('config') . '/autoload/plug.vim'
    if !filereadable(autoload_plug_path)
        silent exe '!curl -fL --create-dirs -o ' . autoload_plug_path .
            \ ' https://raw.github.com/junegunn/vim-plug/master/plug.vim'
        execute 'source ' . fnameescape(autoload_plug_path)
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
