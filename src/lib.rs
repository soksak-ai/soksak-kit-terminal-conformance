//! Build-time facade for `soksak-contract-terminal` conformance.
//!
//! The contract repository owns the corpus, canonical [`ScreenState`], declared reference states,
//! and acceptance assertions. This crate retains the established named test entry points and the
//! [`conformance_suite!`] macro, but it does not create expected state from a renderer.
//!
//! [`judge`] is an Alacritty-backed diagnostic renderer. It remains available for provider probes
//! and carries its existing license notice, but no conformance path uses it as an authority.

pub mod fixtures;
pub mod judge;

pub use judge::{CellSnap, ColorSnap, MIRROR_SCROLLBACK_LINES, ModeSnap, Screen};
pub use soksak_contract_terminal::reference_state;
pub use soksak_contract_terminal::{
    Attrs, Cell, Color, CursorShape, CursorStyle, Fixture, MirrorUnderTest, Modes, Row,
    ScreenState, assert_conforms, assert_cursor_style_conforms, assert_resize_reflow,
};

/// Runs the contract-owned cases against an engine unit's mirror.
///
/// The mirror must expose the canonical state required by
/// [`soksak_contract_terminal::MirrorUnderTest`]. A consumer that needs an adapter defines that
/// adapter in its own test crate, where the trait implementation remains local and explicit.
#[macro_export]
macro_rules! conformance_suite {
    ($mirror:ty) => {
        #[test]
        fn mid_escape_tail_restores_exact_rows() {
            $crate::fixtures::mid_escape_tail_restores_exact_rows::<$mirror>();
        }

        #[test]
        fn cjk_width_survives_mid_char_cut() {
            $crate::fixtures::cjk_width_survives_mid_char_cut::<$mirror>();
        }

        #[test]
        fn alt_screen_state_and_primary_scrollback_restore() {
            $crate::fixtures::alt_screen_state_and_primary_scrollback_restore::<$mirror>();
        }

        #[test]
        fn private_modes_rehydrate_after_ring_window() {
            $crate::fixtures::private_modes_rehydrate_after_ring_window::<$mirror>();
        }

        #[test]
        fn replay_never_answers_queries() {
            $crate::fixtures::replay_never_answers_queries::<$mirror>();
        }

        #[test]
        fn cold_paint_of_alt_screen_carries_the_tui_frame_and_primary_scrollback() {
            $crate::fixtures::cold_paint_of_alt_screen_carries_the_tui_frame_and_primary_scrollback::<
                $mirror,
            >();
        }

        #[test]
        fn dec_line_drawing_box_restores_glyphs() {
            $crate::fixtures::dec_line_drawing_box_restores_glyphs::<$mirror>();
        }

        #[test]
        fn cursor_style_matches_declared_cases() {
            $crate::assert_cursor_style_conforms::<$mirror>();
        }

        #[test]
        fn resize_reflow_preserves_canonical_state() {
            $crate::assert_resize_reflow::<$mirror>();
        }
    };
}
