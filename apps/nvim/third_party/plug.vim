" Warning: this file is autogenerated, see this source instead:
" {{source_file}}
let s:plug_dir = {{ xdg_or_win_cache_dir("vim-plug") | enquote }}
execute 'set runtimepath+=' . fnameescape(s:plug_dir)

call plug#begin(s:plug_dir . '/plugins')
Plug 'tpope/vim-sensible'
Plug 'editorconfig/editorconfig-vim'
Plug 'machakann/vim-highlightedyank'
Plug 'dag/vim-fish'
Plug 'itchyny/lightline.vim'
Plug 'dracula/vim', { 'as': 'dracula' }
Plug 'rust-lang/rust.vim'
Plug 'tpope/vim-commentary'

if executable("node")
    Plug 'neoclide/coc.nvim', {'branch': 'release'}
    runtime! opt_plugin/coc.vim
endif

call plug#end()

" Force tpope/vim-sensible to load now.
" Otherwise it overrides vimrc and init.vim.
runtime! plugin/sensible.vim
