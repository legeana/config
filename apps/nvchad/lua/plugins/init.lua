return {
  {
    "stevearc/conform.nvim",
    -- event = 'BufWritePre', -- uncomment for format on save
    opts = require "configs.conform",
  },

  {
    "nvim-tree/nvim-tree.lua",
    opts = {
      git = { enable = true },
    },
  },

  {
    "nvim-treesitter/nvim-treesitter",
    opts = {
      ensure_installed = require("configs.languages").treesitter,
    },
  },

  {
    "williamboman/mason.nvim",
    --opts = {
    --  ensure_installed = require("configs.languages").mason,
    --},
  },

  {
    "williamboman/mason-lspconfig.nvim",
    opts = {
      ensure_installed = require("configs.languages").lsp,
    },
    -- lazy = false,
  },

  {
    "neovim/nvim-lspconfig",
    config = function()
      require "configs.lspconfig"
    end,
  },
}
