set runtimepath+=$HOME/.config/nvim

set background=dark
"cmap W w
"cmap Q q
syntax on
"set cindent
set hidden
"set mouse=a
set notitle
" установить keymap, чтобы по Ctrl+^ переключался на русский и обратно
"set keymap=russian-jcukenwin
" по умолчанию - латинская раскладка
"set iminsert=0
" по умолчанию - латинская раскладка при поиске
"set imsearch=0
set ignorecase
set smartcase
set hlsearch
set incsearch
"set textwidth=80
" минимальная высота окна пусть будет 0 (по умолчанию - 1)
set winminheight=0
" всегда делать активное окно максимального размера
set noequalalways
set winheight=9999
" установить шрифт Courier New Cyr
set guifont=courier_new:h10:cRUSSIAN
" настраиваю для работы с русскими словами (чтобы w, b, * понимали
" русские слова)
set iskeyword=@,48-57,_,192-255

function! MyRetab()
    set tabstop=4
    retab
    execute '%s/ \+$//gc'
endfunction
