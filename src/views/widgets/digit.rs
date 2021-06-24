use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    symbols,
    widgets::{Block, Widget},
};

use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone)]
pub struct Digit<'a> {
    block: Option<Block<'a>>,
    num: u8,
    symbol: Option<String>,
    style: Style,
}

impl<'a> Default for Digit<'a> {
    fn default() -> Digit<'a> {
        Digit {
            block: None,
            num: 0,
            symbol: None,
            style: Style::default(),
        }
    }
}

impl<'a> Digit<'a> {
    pub fn _block(mut self, block: Block<'a>) -> Digit<'a> {
        self.block = Some(block);
        self
    }

    pub fn skin(mut self, char: char) -> Digit<'a> {
        self.symbol = Some(String::from(char));
        self
    }

    pub fn num(mut self, num: u8) -> Digit<'a> {
        self.num = num;
        self
    }

    pub fn style(mut self, style: Style) -> Digit<'a> {
        self.style = style;
        self
    }
}

struct Painter<'a> {
    buf: &'a mut Buffer,
    sym: &'a str,
}

impl<'a> Painter<'a> {
    pub fn set_repeated_stringn<S>(&mut self, x: u16, y: u16, string: S, width: usize) -> (u16, u16)
    where
        S: AsRef<str>,
    {
        let mut start_x = x;
        let mut width = width;
        loop {
            let (new_x, _) = self
                .buf
                .set_stringn(start_x, y, &string, width, Default::default());
            if new_x == start_x {
                break;
            }
            width = width.wrapping_sub((new_x - start_x) as usize);
            start_x = new_x;
        }

        (start_x, y)
    }

    fn h(&mut self, x1: u16, x2: u16, y: u16) -> (u16, u16) {
        self.set_repeated_stringn(x1, y, self.sym, (x2 - x1) as usize)
    }

    fn v(&mut self, y1: u16, y2: u16, x: u16) {
        for y in y1..y2 {
            self.buf.get_mut(x, y).set_symbol(self.sym);
        }
    }
}

impl<'a> Widget for Digit<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        let digit_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };
        if digit_area.height <= 2 {
            return;
        }
        buf.set_style(digit_area, self.style);

        let sym = self.symbol.as_deref().unwrap_or(symbols::block::FULL);
        let sym_width = sym.width() as u16;

        let (left, right, top, bottom) = (
            digit_area.left(),
            digit_area.right(),
            digit_area.top(),
            digit_area.bottom(),
        );
        let vcenter = digit_area.height / 2 + top;
        let hcenter = digit_area.width / 2 + left;

        let mut p = Painter { sym, buf };

        match self.num {
            0 => {
                let (endx, _) = p.h(left, right, top);
                p.h(left, right, bottom - 1);
                p.v(top, bottom, left);
                p.v(top, bottom, endx - sym_width);
            }
            1 => {
                p.v(top, bottom, hcenter);
            }
            2 => {
                let (endx, _) = p.h(left, right, top);
                p.h(left, right, vcenter);
                p.h(left, right, bottom - 1);
                p.v(top, vcenter, endx - sym_width);
                p.v(vcenter, bottom - 1, left);
            }
            3 => {
                let (endx, _) = p.h(left, right, top);
                p.h(left, right, bottom - 1);
                p.h(left, right, vcenter);
                p.v(top, bottom - 1, endx - sym_width);
            }
            4 => {
                let (endx, _) = p.h(left, right, vcenter);
                p.v(top, vcenter, left);
                p.v(top, bottom, endx - sym_width);
            }
            5 => {
                let (endx, _) = p.h(left, right, top);
                p.v(vcenter, bottom, endx - sym_width);
                p.v(top, vcenter, left);
                p.h(left, right, vcenter);
                p.h(left, right, bottom - 1);
            }
            6 => {
                let (endx, _) = p.h(left, right, top);
                p.h(left, right, vcenter);
                p.h(left, right, bottom - 1);
                p.v(vcenter, bottom, endx - sym_width);
                p.v(top, bottom, left);
            }
            7 => {
                let (endx, _) = p.h(left, right, top);
                p.v(top, bottom, endx - sym_width);
            }
            8 => {
                let (endx, _) = p.h(left, right, vcenter);
                p.h(left, right, top);
                p.h(left, right, bottom - 1);
                p.v(top, bottom, left);
                p.v(top, bottom, endx - sym_width);
            }
            9 => {
                let (endx, _) = p.h(left, right, top);
                p.v(top, bottom, endx - sym_width);
                p.v(top, vcenter, left);
                p.h(left, right, vcenter);
            }
            _ => {}
        }
    }
}
