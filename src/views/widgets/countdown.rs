use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    widgets::{Block, Widget},
};

use super::Digit;

const SEP_CHAR: &str = "●";
const DEFAULT_SKIN: char = '口';

#[derive(Default, Debug, Clone)]
pub struct Countdown<'a> {
    block: Option<Block<'a>>,
    minutes: u8,
    seconds: u8,
    symbol: Option<char>,
    style: Style,
    digit_style: Style,
}

impl<'a> Countdown<'a> {
    #[allow(dead_code)]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn skin(mut self, char: char) -> Self {
        self.symbol = Some(char);
        self
    }

    #[allow(dead_code)]
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn digit_style(mut self, style: Style) -> Self {
        self.digit_style = style;
        self
    }

    pub fn minutes(mut self, minutes: u64) -> Self {
        self.minutes = if minutes > 99 { 99 } else { minutes as u8 };
        self
    }

    pub fn seconds(mut self, seconds: u64) -> Self {
        self.seconds = if seconds > 60 { 60 } else { seconds as u8 };
        self
    }
}

impl<'a> Widget for Countdown<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        if area.height <= 2 {
            return;
        }
        buf.set_style(area, self.style);

        // 4 width-equal digits
        // 2 widht-equal digit gap
        // the left space is for middle gap
        let digit_w = area.width / 5;
        let gap = (area.width - digit_w * 4) / 4;
        let middle_gap = area.width - digit_w * 4 - gap * 2;
        let mut chunks = vec![Rect::new(area.x, area.y, digit_w, area.height)];
        for delta in &[gap, digit_w, middle_gap, digit_w, gap, digit_w] {
            let last = chunks.last().unwrap();
            let rect = Rect::new(last.x + last.width, area.y, *delta, area.height);
            chunks.push(rect);
        }

        let skin = self.symbol.unwrap_or(DEFAULT_SKIN);

        let (h, l) = (self.minutes / 10, self.minutes % 10);
        Digit::default()
            .num(h)
            .skin(skin)
            .style(self.digit_style)
            .render(chunks[0], buf);

        Digit::default()
            .num(l)
            .skin(skin)
            .style(self.digit_style)
            .render(chunks[2], buf);

        let (h, l) = (self.seconds / 10, self.seconds % 10);
        Digit::default()
            .num(h)
            .skin(skin)
            .style(self.digit_style)
            .render(chunks[4], buf);

        Digit::default()
            .num(l)
            .skin(skin)
            .style(self.digit_style)
            .render(chunks[6], buf);

        let sep_area = chunks[3];
        let hcenter = sep_area.x + (sep_area.width - 1) / 2;
        let v1fourth = sep_area.y + sep_area.height / 4;
        let v3fourth = sep_area.y + sep_area.height / 4 * 3;
        buf.get_mut(hcenter, v1fourth)
            .set_symbol(SEP_CHAR)
            .set_style(self.digit_style);
        buf.get_mut(hcenter, v3fourth)
            .set_symbol(SEP_CHAR)
            .set_style(self.digit_style);
    }
}
