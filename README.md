# VK Teams Bot — Rust workspace for VK Teams Bot API (unofficial)

[![github.com](https://github.com/bug-ops/vkteams-bot/actions/workflows/rust.yml/badge.svg)](https://github.com/bug-ops/vkteams-bot/actions)
[![CodeQL](https://github.com/bug-ops/vkteams-bot/actions/workflows/github-code-scanning/codeql/badge.svg)](https://github.com/bug-ops/vkteams-bot/actions/workflows/github-code-scanning/codeql)
[![Unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

**A multifunctional toolkit for working with the VK Teams Bot API in Rust.**

## Workspace Structure

### Main Crates

#### [`vkteams-bot`](crates/vkteams-bot)

The main client crate for the VK Teams Bot API.

- Create bots
- Send and receive messages
- Work with chats and files
- Usage examples: [`examples`](crates/vkteams-bot/examples)

#### [`vkteams-bot-mcp`](crates/vkteams-bot-mcp)

An MCP (Model Context Protocol) server for integrating VK Teams bots with LLM agents, automation, and external services via a universal protocol (stdin/stdout, JSON).

#### [`vkteams-bot-cli`](crates/vkteams-bot-cli)

A powerful CLI tool for bot management:

- Sending messages
- Working with files
- Event monitoring
- Task scheduler
- Interactive setup
- Shell completion and much more

#### [`vkteams-bot-macros`](crates/vkteams-bot-macros)

A set of macros to simplify working with the VK Teams Bot API (used internally).

---

## Documentation

- [VK Teams Bot API (official documentation)](https://teams.vk.com/botapi/?lang=en)
- [MCP protocol documentation](https://modelcontextprotocol.io/specification/2025-03-26)
- [Documentation on docs.rs](https://docs.rs/vkteams-bot/latest/vkteams_bot/)
