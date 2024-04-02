local M = {}

-- lsp: https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md
local packages = {
  { lsp = "clangd", ts = {"c", "cpp"} },
  { lsp = "cssls", ts = {"css"} },
  { lsp = "gopls", ts = {"go"} },
  { lsp = "html", ts = {"html"} },
  -- { lsp = "java_language_server", ts = {"java"} },
  { lsp = "pylsp", ts = {"python"} },
  { lsp = "rust_analyzer", ts = {"rust"} },
  { lsp = "tsserver", ts = {"typescript"} },
}

M.lsp = vim.tbl_map(function(e) return e.lsp end, packages)
M.treesitter = vim.tbl_flatten(vim.tbl_map(function(e) return e.ts end, packages))

return M
