//! Named compatibility entry points for the terminal contract's seven declared cases.
//!
//! The corpus, expected [`ScreenState`](crate::ScreenState), and assertions live in
//! `soksak-contract-terminal`. These functions retain the kit's public test names while delegating
//! every verdict to that contract. No renderer in this crate supplies expected state.

use crate::{Fixture, MirrorUnderTest, assert_conforms};

pub fn mid_escape_tail_restores_exact_rows<M: MirrorUnderTest>() {
    assert_conforms::<M>(Fixture::MidEscapeTail);
}

pub fn cjk_width_survives_mid_char_cut<M: MirrorUnderTest>() {
    assert_conforms::<M>(Fixture::CjkWidth);
}

pub fn alt_screen_state_and_primary_scrollback_restore<M: MirrorUnderTest>() {
    assert_conforms::<M>(Fixture::AltScreen);
}

pub fn private_modes_rehydrate_after_ring_window<M: MirrorUnderTest>() {
    assert_conforms::<M>(Fixture::PrivateModes);
}

pub fn replay_never_answers_queries<M: MirrorUnderTest>() {
    assert_conforms::<M>(Fixture::ReplayGuard);
}

pub fn cold_paint_of_alt_screen_carries_the_tui_frame_and_primary_scrollback<M: MirrorUnderTest>() {
    assert_conforms::<M>(Fixture::ColdPaintAlt);
}

pub fn dec_line_drawing_box_restores_glyphs<M: MirrorUnderTest>() {
    assert_conforms::<M>(Fixture::DecLineDrawing);
}
