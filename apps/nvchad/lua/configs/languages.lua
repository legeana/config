local M = {}

-- TODO: Consider using mason-lspconfig.nvim instead.
-- Example: https://github.com/pynappo/dotfiles/blob/daa6c778be8e3a20f574b5a6f8ea8eda1fdcd6cd/.config/nvim/lua/pynappo/plugins/lsp.lua#L46-L53

-- lsp: https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md
local packages = {
  { lsp = "clangd", mason = "clangd", ts = {"c", "cpp"} },
  { lsp = "cssls", mason = "css-lsp", ts = {"css"} },
  { lsp = "gopls", mason = "gopls", ts = {"go"} },
  { lsp = "html", mason = "html-lsp", ts = {"html"} },
  -- { lsp = "java_language_server", mason = "java-language-server", ts = {"java"} },
  { lsp = "pylsp", mason = "python-lsp-server", ts = {"python"} },
  { lsp = "rust_analyzer", mason = "rust-analyzer", ts = {"rust"} },
  { lsp = "tsserver", mason = "typescript-language-server", ts = {"typescript"} },
}

M.lsp = vim.tbl_map(function(e) return e.lsp end, packages)
M.mason = vim.tbl_map(function(e) return e.mason end, packages)
M.treesitter = vim.tbl_flatten(vim.tbl_map(function(e) return e.ts end, packages))

return M
