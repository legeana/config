return {
  {
    "stevearc/conform.nvim",
    config = function()
      require "configs.conform"
    end,
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
    lazy = false,
  },

  {
    "neovim/nvim-lspconfig",
    config = function()
      require("nvchad.configs.lspconfig").defaults()
      require "configs.lspconfig"
    end,
  },
}
