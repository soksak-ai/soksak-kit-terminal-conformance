//! Alacritty-backed diagnostic renderer.
//!
//! [`Screen`] 은 바이트를 진짜로 그려서 화면 상태(그리드·스크롤백·모드·커서)를 읽고, 터미널이
//! PTY 에 되쓰려는 응답(DA1/DSR 답)을 [`ReplyTap`] 으로 포획한다. Provider probe가 렌더링 결과와
//! 응답을 관찰할 때 쓰며, contract conformance의 expected state를 만들지 않는다.
//!
//! Correctness is defined by the declared reference states in `soksak-contract-terminal`. This
//! renderer is one provider implementation and is never a universal authority.
//!
//! 스냅샷 타입([`ColorSnap`]·[`CellSnap`]·[`ModeSnap`])은 이 diagnostic surface의 관찰 형식이다.

use std::sync::{Arc, Mutex};

use alacritty_terminal::event::{Event, EventListener};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::index::{Column, Line};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::test::TermSize;
use alacritty_terminal::term::{Config, Term, TermMode};
use alacritty_terminal::vte::ansi::{Color, NamedColor, Processor};

/// Diagnostic renderer scrollback capacity in rows.
pub const MIRROR_SCROLLBACK_LINES: usize = 1000;

// ── Diagnostic snapshot types ────────────────────────────────────────────────

/// 색 스냅샷 — 엔진 타입을 밖으로 새지 않게 자체 표현으로 고정한다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColorSnap {
    Default,
    Named(u8),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// 셀 한 칸의 비교 가능한 스냅샷. wide 문자는 스냅 1개(점유 2칸)로 나오고 스페이서 셀은
/// 생략된다 — "폭"은 `wide` 가 진실이다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CellSnap {
    pub ch: char,
    pub fg: ColorSnap,
    pub bg: ColorSnap,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
    pub strikeout: bool,
    pub hidden: bool,
    pub wide: bool,
}

/// 복원 대상 private mode 집합의 스냅샷(rehydrate 가 재현해야 하는 전부).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModeSnap {
    pub bracketed_paste: bool,
    pub app_cursor: bool,
    pub app_keypad: bool,
    pub mouse_click: bool,
    pub mouse_drag: bool,
    pub mouse_motion: bool,
    pub sgr_mouse: bool,
    pub utf8_mouse: bool,
    pub focus_in_out: bool,
    pub alternate_scroll: bool,
    pub show_cursor: bool,
    pub line_wrap: bool,
    pub insert: bool,
}

/// Diagnostic renderer cell before spacer normalization.
struct GridCell {
    ch: char,
    fg: ColorSnap,
    bg: ColorSnap,
    bold: bool,
    dim: bool,
    italic: bool,
    underline: bool,
    inverse: bool,
    strikeout: bool,
    hidden: bool,
    wide: bool,
    spacer: bool,
}

// ── 이벤트 프록시 — 터미널이 PTY 에 쓰려는 응답을 포획한다 ─────────────────────

#[derive(Clone, Default)]
struct ReplyTap(Arc<Mutex<Vec<String>>>);

impl EventListener for ReplyTap {
    fn send_event(&self, event: Event) {
        if let Event::PtyWrite(text) = event {
            self.0.lock().unwrap_or_else(|e| e.into_inner()).push(text);
        }
    }
}

// ── Screen — Alacritty diagnostic renderer ──────────────────────────────────

/// Headless terminal used for provider diagnostics. Contract conformance does not derive expected
/// state from this renderer.
pub struct Screen {
    term: Term<ReplyTap>,
    parser: Processor,
    replies: Arc<Mutex<Vec<String>>>,
    cols: u16,
    rows: u16,
}

impl Screen {
    pub fn new(cols: u16, rows: u16) -> Self {
        let tap = ReplyTap::default();
        let replies = tap.0.clone();
        let config = Config {
            scrolling_history: MIRROR_SCROLLBACK_LINES,
            ..Config::default()
        };
        let term = Term::new(config, &TermSize::new(cols as usize, rows as usize), tap);
        Screen {
            term,
            parser: Processor::new(),
            replies,
            cols,
            rows,
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        self.parser.advance(&mut self.term, bytes);
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.term
            .resize(TermSize::new(cols as usize, rows as usize));
    }

    pub fn cols(&self) -> u16 {
        self.cols
    }

    pub fn rows(&self) -> u16 {
        self.rows
    }

    /// Bytes this diagnostic terminal attempted to return to the PTY.
    pub fn captured_replies(&self) -> Vec<String> {
        self.replies
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn alt_active(&self) -> bool {
        self.term.mode().contains(TermMode::ALT_SCREEN)
    }

    /// 커서 위치(화면 기준 0-base row, col).
    pub fn cursor(&self) -> (usize, usize) {
        let p = self.term.grid().cursor.point;
        (p.line.0.max(0) as usize, p.column.0)
    }

    pub fn modes(&self) -> ModeSnap {
        let m = self.term.mode();
        ModeSnap {
            bracketed_paste: m.contains(TermMode::BRACKETED_PASTE),
            app_cursor: m.contains(TermMode::APP_CURSOR),
            app_keypad: m.contains(TermMode::APP_KEYPAD),
            mouse_click: m.contains(TermMode::MOUSE_REPORT_CLICK),
            mouse_drag: m.contains(TermMode::MOUSE_DRAG),
            mouse_motion: m.contains(TermMode::MOUSE_MOTION),
            sgr_mouse: m.contains(TermMode::SGR_MOUSE),
            utf8_mouse: m.contains(TermMode::UTF8_MOUSE),
            focus_in_out: m.contains(TermMode::FOCUS_IN_OUT),
            alternate_scroll: m.contains(TermMode::ALTERNATE_SCROLL),
            show_cursor: m.contains(TermMode::SHOW_CURSOR),
            line_wrap: m.contains(TermMode::LINE_WRAP),
            insert: m.contains(TermMode::INSERT),
        }
    }

    /// 보이는 화면(위→아래). 행 끝의 스타일 없는 공백은 잘라 비교를 안정화한다.
    pub fn visible_rows(&self) -> Vec<Vec<CellSnap>> {
        (0..self.rows as i32).map(|l| self.snap_row(l)).collect()
    }

    /// 스크롤백(가장 오래된 것부터). 화면 위로 밀려난 행들만.
    pub fn history_rows(&self) -> Vec<Vec<CellSnap>> {
        let hist = self.term.grid().history_size() as i32;
        (-hist..0).map(|l| self.snap_row(l)).collect()
    }

    /// 행 텍스트만(스타일 무시) — 마커 존재 단언용.
    pub fn text_of(rows: &[Vec<CellSnap>]) -> Vec<String> {
        rows.iter()
            .map(|r| r.iter().map(|c| c.ch).collect())
            .collect()
    }

    // 한 행을 CellSnap 벡터로. spacer 는 생략(폭은 wide 가 진실), 꼬리의 무스타일 공백은
    // 잘라낸다(미러 직렬화기의 꼬리 생략과 같은 기준).
    fn snap_row(&self, line: i32) -> Vec<CellSnap> {
        let mut out: Vec<CellSnap> = Vec::new();
        for cell in self.line_cells(line) {
            if cell.spacer {
                continue;
            }
            out.push(CellSnap {
                ch: cell.ch,
                fg: cell.fg,
                bg: cell.bg,
                bold: cell.bold,
                dim: cell.dim,
                italic: cell.italic,
                underline: cell.underline,
                inverse: cell.inverse,
                strikeout: cell.strikeout,
                hidden: cell.hidden,
                wide: cell.wide,
            });
        }
        while out.last().is_some_and(|c| {
            c.ch == ' '
                && c.fg == ColorSnap::Default
                && c.bg == ColorSnap::Default
                && !(c.bold
                    || c.dim
                    || c.italic
                    || c.underline
                    || c.inverse
                    || c.strikeout
                    || c.hidden)
        }) {
            out.pop();
        }
        out
    }

    // 한 행(line index; 음수 = 스크롤백)을 그리드에서 읽는다. 길이는 항상 `cols`.
    fn line_cells(&self, line: i32) -> Vec<GridCell> {
        let grid = self.term.grid();
        let row = &grid[Line(line)];
        (0..self.cols as usize)
            .map(|col| {
                let cell = &row[Column(col)];
                GridCell {
                    ch: cell.c,
                    fg: snap_color(&cell.fg),
                    bg: snap_color(&cell.bg),
                    bold: cell.flags.contains(Flags::BOLD),
                    dim: cell.flags.contains(Flags::DIM),
                    italic: cell.flags.contains(Flags::ITALIC),
                    underline: cell.flags.intersects(Flags::ALL_UNDERLINES),
                    inverse: cell.flags.contains(Flags::INVERSE),
                    strikeout: cell.flags.contains(Flags::STRIKEOUT),
                    hidden: cell.flags.contains(Flags::HIDDEN),
                    wide: cell.flags.contains(Flags::WIDE_CHAR),
                    spacer: cell
                        .flags
                        .intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER),
                }
            })
            .collect()
    }
}

fn snap_color(color: &Color) -> ColorSnap {
    match color {
        Color::Named(NamedColor::Foreground) | Color::Named(NamedColor::Background) => {
            ColorSnap::Default
        }
        Color::Named(n) => ColorSnap::Named(*n as u8),
        Color::Indexed(i) => ColorSnap::Indexed(*i),
        Color::Spec(rgb) => ColorSnap::Rgb(rgb.r, rgb.g, rgb.b),
    }
}
