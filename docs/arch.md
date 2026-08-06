# Architecture

The code is broken down into two layers:

## ui

The view layer, built with gpui. `app::run` is the entry point `main` calls; it creates the `Application` and opens the root window. Each view gets its own file and implements `Render`. No domain logic lives here — views call through to the feature layer.

## feature

Where all domain logic lives. The ui layer calls through to feature. Contains no gpui types.
