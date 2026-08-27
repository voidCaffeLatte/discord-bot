# Discord Bot

A feature-rich Discord bot written in Rust that brings AI-powered character chat, image generation, and content discovery to your server.

## Features

### AI Chat (`/ai-chat`)

Have one-on-one conversations with customizable AI characters. Each character has a name, title, and personality traits defined in a configuration file.

- Persistent chat history per user per character
- Web reference citations included in AI responses
- Daily usage limit per user

### AI Conversation (`/ai-conversation`)

Start a group conversation between two or more AI characters on a given theme and watch them interact with each other in-character.

### AI Image Generation (`/ai-image`)

Generate images from text prompts. Daily usage limit per user.

### Random YouTube Video (`/random-you-tube-video`)

Fetch random videos from a YouTube channel by providing its handle (e.g. `@channel`). Displays title, view count, duration, publish date, and link.

### Random Twitch Clip (`/random-twitch-clip`)

Fetch random clips from a Twitch streamer by providing their user ID, with an optional amount to fetch more than one at a time. Displays title, view count, duration, creation date, and link.

### Utility

- `/ping` — Check bot status
- `/choices` — Randomly pick from a list of user-provided options

## Architecture

The project follows clean architecture principles with a Cargo workspace made up of the root binary crate and five member crates:

```
src/              Presentation layer (Discord command & modal handlers)
application/      Use cases and gateway/repository trait definitions
domain/           Business models and value objects
infrastructure/   Concrete implementations (API clients, in-memory repositories)
common/           Shared utilities (i18n, caching, collections)
lib/gemini/       Standalone client library for the Google Gemini API
```

Key design decisions:

- **Dependency inversion** — Traits are defined in `application/`, implementations live in `infrastructure/`
- **In-memory storage** — All state (chat history, activity counters) is currently held in `DashMap` as a provisional implementation without transaction management. A migration to an RDBMS with proper transaction support is planned.
- **Localization** — All user-facing strings are managed via Fluent (`.ftl` files) with Japanese as the primary locale

## Prerequisites

- **Nix** with flakes enabled, and **direnv** — running `direnv allow` in the project root enters the development shell defined in `flake.nix`, which provides the Rust toolchain, `rust-analyzer`, the native build dependencies (`pkg-config`, `openssl`, `libopus`), and the Railway CLI
- Without Nix, install **Rust** 1.96.1 (pinned via `rust-toolchain.toml`) and the native dependencies yourself
- API keys for the external services (see [Configuration](#configuration))

## Configuration

### Environment Variables

`.envrc` loads the environment variables through direnv, so copy the template and fill in your credentials:

```bash
cp .env.template .env
```

`.env` is loaded first, and `.env.development` is loaded afterwards if it exists, so you can keep development-only overrides there without touching `.env`. All `.env*` files except `.env.template` are gitignored.

| Variable | Description |
|---|---|
| `DISCORD_TOKEN` | Discord bot token |
| `DISCORD_BOT_APPLICATION_ID` | Discord application ID |
| `GEMINI_API_KEY` | Google Gemini API key |
| `YOU_TUBE_DATA_API_KEY` | YouTube Data API v3 key |
| `TWITCH_APP_CLIENT_ID` | Twitch application client ID |
| `TWITCH_APP_CLIENT_SECRET` | Twitch application client secret |
| `LOGGING_WEB_HOOK_URL` | Discord webhook URL for error logging |

### Character Definition

AI characters are configured in `resource/character.toml`. Copy the template to get started:

```bash
cp resource/character.toml.template resource/character.toml
```

Each character entry has the following fields:

```toml
[[characters]]
name = "Character Name"
title = "Title"
characteristics = ["trait1", "trait2", "trait3"]
```

### Locale Override

You can override any localized string by creating `resource/locale/ja-JP.override.ftl`. This file is gitignored and will not be committed.

## Building & Running

### Local

With direnv active, the toolchain and the environment variables from `.env` are already loaded in the shell:

```bash
cargo build --release
cargo run
```

Without direnv, enter the development shell manually with `nix develop` and load the environment variables yourself.

### Docker

```bash
docker build -t discord-bot .
docker run --env-file .env discord-bot
```

### Railway

The project includes a `.railwayignore` for deployment on [Railway](https://railway.app/). The Railway CLI is available inside the Nix development shell.

## Tech Stack

| Category | Technology |
|---|---|
| Language | Rust (Edition 2024) |
| Discord Library | Serenity |
| Async Runtime | Tokio |
| AI Text Generation | Google Gemini API (`gemini-3.6-flash`) |
| AI Image Generation | Google Gemini API (`gemini-3.1-flash-image-preview`) |
| HTTP Client | reqwest |
| Data Storage | In-memory (DashMap) |
| Localization | Fluent (Japanese) |
| Logging | tracing / tracing-subscriber |
| Development Environment | Nix flake + direnv |

---

<sub>This document was generated by Claude Opus 4.6 on [Claude Code](https://claude.com/claude-code) and reviewed and adjusted by [@voidCaffeLatte](https://github.com/voidCaffeLatte).</sub>
