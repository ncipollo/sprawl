# Architecture

The code is broken down into three layers:

## cli

A thin clap router; `cli::router::run` is the entry point `main` calls. No domain logic lives here — it dispatches to `ui::app::run` for the gui (the default, argument-less invocation) and to the `feature::info` pages for `--info`.

## ui

The view layer, built with gpui. `app::run` creates the `Application` and opens the root window. Each view gets its own file and implements `Render`. No domain logic lives here — views call through to the feature layer.

## feature

Where all domain logic lives. The ui layer calls through to feature. Contains no gpui types.

## The config-script pipeline

Sections are defined by user JavaScript files in `~/.sprawl/default` (scaffolded with examples on first run). `feature::config` locates and lists them; `feature::script` evaluates one in a sandboxed boa engine whose only injected capability is a `shell` function, and deserializes the script's completion value into the section schema; `feature::section`'s store caches results per script with a stale-while-revalidate policy; and `ui::section_pane` renders the cached items and drives refreshes.
