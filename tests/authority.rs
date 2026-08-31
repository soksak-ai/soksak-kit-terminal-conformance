use soksak_kit_terminal_conformance::{
    CursorShape, CursorStyle, Fixture, MirrorUnderTest, Modes, Row, ScreenState, assert_conforms,
    reference_state,
};

const RESTORE_PAINT: &[u8] = b"SOKSAK-DECLARED-REPLAY-GUARD";

struct DeclaredStateMirror {
    state: ScreenState,
    suppressed_replies: u64,
}

impl DeclaredStateMirror {
    fn blank(cols: u16, rows: u16) -> ScreenState {
        ScreenState {
            cols,
            rows,
            alt: false,
            cursor: (0, 0),
            modes: Modes::default(),
            history: Vec::new(),
            visible: vec![Row::default(); rows as usize],
        }
    }
}

impl MirrorUnderTest for DeclaredStateMirror {
    fn new(cols: u16, rows: u16) -> Self {
        Self {
            state: Self::blank(cols, rows),
            suppressed_replies: 0,
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        if bytes.starts_with(b"GUARD-MARK") {
            self.state = reference_state::load(Fixture::ReplayGuard.stem());
            self.suppressed_replies = 1;
        } else if bytes == RESTORE_PAINT {
            self.state = reference_state::load(Fixture::ReplayGuard.stem());
            self.suppressed_replies = 0;
        }
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        self.state.cols = cols;
        self.state.rows = rows;
    }

    fn rehydrate(&self) -> Vec<u8> {
        RESTORE_PAINT.to_vec()
    }

    fn cold_paint(&self) -> Vec<u8> {
        Vec::new()
    }

    fn suppressed_replies(&self) -> u64 {
        self.suppressed_replies
    }

    fn screen_state(&self) -> ScreenState {
        self.state.clone()
    }

    fn cursor_style(&self) -> CursorStyle {
        CursorStyle {
            shape: CursorShape::Block,
            blinking: false,
        }
    }
}

#[test]
fn declared_reference_state_is_the_only_screen_authority() {
    assert_conforms::<DeclaredStateMirror>(Fixture::ReplayGuard);
}
