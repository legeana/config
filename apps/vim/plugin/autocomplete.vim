if !exists('+autocomplete')
    finish
endif

" Show autocomplete automatically.
set autocomplete

" Limit to 5 suggestions.
set complete=.^5,w^5,b^5,u^5

" TODO: Figure this out.
" set completeopt=popup

" Map Tab / Shift-Tab.
inoremap <silent><expr> <Tab>   pumvisible() ? "\<C-n>" : "\<Tab>"
inoremap <silent><expr> <S-Tab> pumvisible() ? "\<C-p>" : "\<S-Tab>"
