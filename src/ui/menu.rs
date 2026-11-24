//! The application menu bar.

use gpui::{App, KeyBinding, Menu, MenuItem, actions};

actions!(sprawl, [Quit, Refresh, PreviousTab, NextTab]);

/// The application menu bar. The first menu is the macOS application menu.
pub fn menus() -> Vec<Menu> {
    vec![
        Menu {
            name: "Sprawl".into(),
            items: vec![MenuItem::action("Quit Sprawl", Quit)],
        },
        Menu {
            name: "View".into(),
            items: vec![
                MenuItem::action("Refresh", Refresh),
                MenuItem::action("Previous Tab", PreviousTab),
                MenuItem::action("Next Tab", NextTab),
            ],
        },
    ]
}

/// Registers menu actions and installs the menu bar. `Refresh`, `PreviousTab`
/// and `NextTab` are bound here but handled by `ui::app`, which registers the listener once the window's
/// root view exists.
pub fn init(cx: &mut App) {
    cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
    cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
    cx.bind_keys([KeyBinding::new("cmd-r", Refresh, None)]);
    cx.bind_keys([KeyBinding::new("cmd-shift-[", PreviousTab, None)]);
    cx.bind_keys([KeyBinding::new("cmd-shift-]", NextTab, None)]);
    cx.set_menus(menus());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menus_has_a_sprawl_menu_with_a_quit_item_and_a_view_menu_with_refresh_and_tab_items() {
        let menus = menus();

        assert_eq!(menus.len(), 2);
        assert_eq!(menus[0].name, "Sprawl");
        assert_eq!(menus[0].items.len(), 1);
        match &menus[0].items[0] {
            MenuItem::Action { name, .. } => assert_eq!(name, "Quit Sprawl"),
            _ => panic!("expected a Quit Sprawl action item"),
        }
        assert_eq!(menus[1].name, "View");
        let view_items: Vec<_> = menus[1]
            .items
            .iter()
            .map(|item| match item {
                MenuItem::Action { name, .. } => name.to_string(),
                _ => panic!("expected only action items in the View menu"),
            })
            .collect();
        assert_eq!(view_items, ["Refresh", "Previous Tab", "Next Tab"]);
    }
}
