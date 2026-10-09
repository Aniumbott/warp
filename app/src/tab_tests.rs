use std::collections::HashMap;

use warpui::platform::keyboard::KeyCode;

use super::{
    SelectedTabColor, ShortcutModifierKind, TAB_ACTIVATE_BINDING_NAMES,
    TAB_ACTIVATE_LAST_BINDING_NAME, TabShortcutModifierState, next_tab_color,
    tab_activate_binding_name, tab_group_menu_entry_flags,
};
use crate::tab::TabColor;
use crate::ui_components::color_dot::TAB_COLOR_OPTIONS;
use crate::workspace::tab_group::{TabGroup, TabGroupId};

/// Build a `tab_groups` map containing exactly the given group ids.
fn groups(ids: &[TabGroupId]) -> HashMap<TabGroupId, TabGroup> {
    ids.iter()
        .map(|id| {
            let mut group = TabGroup::new();
            group.id = *id;
            (*id, group)
        })
        .collect()
}

// GH-13073: a tab that is the sole member of its group must NOT be offered
// "New group with tab" (it would just recreate an identical single-tab group);
// it offers "Remove from group" instead.
#[test]
fn sole_member_of_group_hides_new_group_and_offers_remove() {
    let gid = TabGroupId::new();
    let (show_new_group, _show_move_to_group, show_remove_from_group) =
        tab_group_menu_entry_flags(Some(gid), &groups(&[gid]), /* is_only_member */ true);

    assert!(
        !show_new_group,
        "the sole member of a group should not offer 'New group with tab'"
    );
    assert!(
        show_remove_from_group,
        "a tab in a group should offer 'Remove from group'"
    );
}

#[test]
fn tab_shortcut_modifier_state_clear_reports_whether_state_changed() {
    let mut state = TabShortcutModifierState::new();

    assert!(!state.clear_held_keys());

    assert!(state.held_keys.insert(KeyCode::SuperLeft));
    assert!(state.held_kinds().is_empty());
    assert!(state.reveal_key_if_held(KeyCode::SuperLeft));
    assert_eq!(
        state.held_kinds(),
        [ShortcutModifierKind::Super].into_iter().collect()
    );

    assert!(state.clear_held_keys());
    assert!(state.held_kinds().is_empty());
    assert!(!state.clear_held_keys());
}

#[test]
fn tab_shortcut_modifier_state_only_reveals_keys_that_remain_held() {
    let mut state = TabShortcutModifierState::new();

    assert!(!state.reveal_key_if_held(KeyCode::SuperLeft));

    assert!(state.held_keys.insert(KeyCode::SuperLeft));
    assert!(state.held_keys.remove(&KeyCode::SuperLeft));
    assert!(!state.reveal_key_if_held(KeyCode::SuperLeft));
    assert!(state.held_kinds().is_empty());
}

#[test]
fn tab_activate_binding_name_prefers_numbered_binding_for_final_tab() {
    assert_eq!(
        tab_activate_binding_name(2, 3),
        Some(TAB_ACTIVATE_BINDING_NAMES[2])
    );
    assert_eq!(
        tab_activate_binding_name(7, 8),
        Some(TAB_ACTIVATE_BINDING_NAMES[7])
    );
}

#[test]
fn tab_activate_binding_name_uses_last_tab_binding_beyond_numbered_tabs() {
    assert_eq!(
        tab_activate_binding_name(8, 9),
        Some(TAB_ACTIVATE_LAST_BINDING_NAME)
    );
    assert_eq!(
        tab_activate_binding_name(9, 10),
        Some(TAB_ACTIVATE_LAST_BINDING_NAME)
    );
}

#[test]
fn tab_activate_binding_name_omits_unbound_and_out_of_bounds_tabs() {
    assert_eq!(tab_activate_binding_name(8, 10), None);
    assert_eq!(tab_activate_binding_name(10, 10), None);
    assert_eq!(tab_activate_binding_name(0, 0), None);
}

// GH-13073 follow-up: a tab that shares a group with siblings SHOULD still be
// offered "New group with tab" so it can be pulled out into its own new group
// (à la Chrome), and it offers "Remove from group" as well.
#[test]
fn grouped_tab_with_siblings_offers_new_group_and_remove() {
    let gid = TabGroupId::new();
    let (show_new_group, _show_move_to_group, show_remove_from_group) =
        tab_group_menu_entry_flags(Some(gid), &groups(&[gid]), /* is_only_member */ false);

    assert!(
        show_new_group,
        "a grouped tab with siblings should still offer 'New group with tab'"
    );
    assert!(
        show_remove_from_group,
        "a grouped tab should offer 'Remove from group'"
    );
}

// An ungrouped tab always offers "New group with tab" and never offers
// "Remove from group". `is_only_member` is irrelevant when ungrouped.
#[test]
fn ungrouped_tab_offers_new_group_and_hides_remove() {
    let (show_new_group, _show_move_to_group, show_remove_from_group) =
        tab_group_menu_entry_flags(None, &HashMap::new(), /* is_only_member */ false);

    assert!(
        show_new_group,
        "an ungrouped tab should offer 'New group with tab'"
    );
    assert!(
        !show_remove_from_group,
        "an ungrouped tab should not offer 'Remove from group'"
    );
}

// "Move to group" should only appear when a group other than the tab's own
// exists — for both grouped and ungrouped tabs.
#[test]
fn move_to_group_only_shown_when_other_groups_exist() {
    let own = TabGroupId::new();
    let other = TabGroupId::new();

    // Grouped tab whose group is the only one: no other groups to move to.
    let (_n, move_only_own, _r) = tab_group_menu_entry_flags(Some(own), &groups(&[own]), true);
    assert!(!move_only_own);

    // Grouped tab with another group present: offer "Move to group".
    let (_n, move_with_other, _r) =
        tab_group_menu_entry_flags(Some(own), &groups(&[own, other]), true);
    assert!(move_with_other);

    // Ungrouped tab with an existing group: offer "Move to group".
    let (_n, move_ungrouped, _r) = tab_group_menu_entry_flags(None, &groups(&[other]), false);
    assert!(move_ungrouped);
}

#[test]
fn next_tab_color_follows_the_canonical_palette_and_clears_after_the_last_color() {
    assert_eq!(
        next_tab_color(None),
        SelectedTabColor::Color(TAB_COLOR_OPTIONS[0])
    );
    for adjacent_colors in TAB_COLOR_OPTIONS.windows(2) {
        assert_eq!(
            next_tab_color(Some(adjacent_colors[0])),
            SelectedTabColor::Color(adjacent_colors[1])
        );
    }
    let last_color = TAB_COLOR_OPTIONS
        .last()
        .copied()
        .expect("the canonical tab color palette should not be empty");
    assert_eq!(next_tab_color(Some(last_color)), SelectedTabColor::Cleared);
    assert_eq!(
        next_tab_color(SelectedTabColor::Cleared.resolve(None)),
        SelectedTabColor::Color(TAB_COLOR_OPTIONS[0])
    );
    assert_eq!(
        next_tab_color(Some(TabColor::White)),
        SelectedTabColor::Color(TAB_COLOR_OPTIONS[0])
    );
}

#[test]
fn next_tab_color_cycles_through_all_twelve_colors_then_clears() {
    assert_eq!(TAB_COLOR_OPTIONS.len(), 12);
    let mut current = None;
    let mut visited = Vec::new();
    loop {
        match next_tab_color(current) {
            SelectedTabColor::Color(color) => {
                visited.push(color);
                current = Some(color);
            }
            SelectedTabColor::Cleared => break,
            SelectedTabColor::Unset => panic!("next_tab_color never returns Unset"),
        }
    }
    assert_eq!(visited, TAB_COLOR_OPTIONS.to_vec());
}

#[test]
fn tab_color_deserializes_legacy_values_and_round_trips_bright_ones() {
    assert_eq!(
        serde_yaml::from_str::<TabColor>("red").unwrap(),
        TabColor::Red
    );
    assert_eq!(
        serde_yaml::from_str::<TabColor>("white").unwrap(),
        TabColor::White
    );
    for color in TAB_COLOR_OPTIONS {
        let yaml = serde_yaml::to_string(&color).unwrap();
        assert_eq!(serde_yaml::from_str::<TabColor>(&yaml).unwrap(), color);
    }
    assert_eq!(
        serde_yaml::to_string(&TabColor::BrightRed)
            .unwrap()
            .trim_start_matches("---")
            .trim(),
        "bright_red"
    );
    assert_eq!(
        serde_yaml::from_str::<TabColor>("bright_red").unwrap(),
        TabColor::BrightRed
    );
}

#[test]
fn selected_tab_color_yaml_round_trips_and_reads_legacy_form() {
    // The form sqlite persists for `SelectedTabColor`.
    for selected in [
        SelectedTabColor::Unset,
        SelectedTabColor::Cleared,
        SelectedTabColor::Color(TabColor::Blue),
        SelectedTabColor::Color(TabColor::BrightMagenta),
    ] {
        let yaml = serde_yaml::to_string(&selected).unwrap();
        assert_eq!(
            serde_yaml::from_str::<SelectedTabColor>(&yaml).unwrap(),
            selected
        );
    }
    // Values written before bright colors existed.
    let legacy = serde_yaml::to_string(&SelectedTabColor::Color(TabColor::Red)).unwrap();
    assert!(legacy.contains("red"), "unexpected form: {legacy}");
    assert_eq!(
        serde_yaml::from_str::<SelectedTabColor>(&legacy).unwrap(),
        SelectedTabColor::Color(TabColor::Red)
    );
}

#[test]
fn tab_color_display_keeps_legacy_labels_and_labels_bright_variants() {
    assert_eq!(TabColor::Red.to_string(), "Red");
    assert_eq!(TabColor::BrightRed.to_string(), "Bright Red");
    assert_eq!(<&'static str>::from(TabColor::BrightRed), "bright_red");
    assert_eq!(
        "bright_red".parse::<TabColor>().unwrap(),
        TabColor::BrightRed
    );
    assert_eq!("RED".parse::<TabColor>().unwrap(), TabColor::Red);
}
