//! Text layout helpers for the information screens.

use crate::{Frame, WHITE, YELLOW};

/// A simple typewriter: successive lines of 8-pixel text with a left margin.
pub struct Text {
    pub x: i16,
    pub y: i16,
    pub line_height: i16,
}

impl Text {
    pub fn new(x: i16, y: i16) -> Self {
        Text {
            x,
            y,
            line_height: 10,
        }
    }

    pub fn line(&mut self, f: &mut Frame, s: &str) {
        f.text(self.x, self.y, s, WHITE);
        self.y += self.line_height;
    }

    /// "Label: value" with the value drawn in a second colour.
    pub fn pair(&mut self, f: &mut Frame, label: &str, value: &str) {
        f.text(self.x, self.y, label, WHITE);
        f.text(
            self.x + 8 * label.chars().count() as i16,
            self.y,
            value,
            YELLOW,
        );
        self.y += self.line_height;
    }

    pub fn blank(&mut self) {
        self.y += self.line_height / 2;
    }

    /// Word-wrap a paragraph to `cols` characters per line.
    pub fn paragraph(&mut self, f: &mut Frame, s: &str, cols: usize) {
        let mut line = String::new();
        for word in s.split_whitespace() {
            if !line.is_empty() && line.len() + 1 + word.len() > cols {
                self.line(f, &line);
                line.clear();
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            self.line(f, &line);
        }
    }
}

/// Screen title with the underline the original draws.
pub fn heading(f: &mut Frame, s: &str) {
    f.text_centred(6, s, WHITE);
    f.line(0, 18, crate::WIDTH - 1, 18, WHITE);
}

/// Format tenths of a credit as "12.3 Cr".
pub fn credits(tenths: u32) -> String {
    format!("{}.{} Cr", tenths / 10, tenths % 10)
}

/// Format tenths of a light year as "7.0 LY".
pub fn light_years(tenths: u16) -> String {
    format!("{}.{} LY", tenths / 10, tenths % 10)
}
