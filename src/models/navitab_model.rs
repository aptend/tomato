#[derive(Debug, PartialEq, Eq)]
pub enum TabType {
    Inventory = 0,
    Tomato = 1,
    Statistics = 2,
}

pub struct NavitabModel {
    pub titles: Vec<String>,
    pub select: usize,
}

impl NavitabModel {
    pub fn new() -> NavitabModel {
        let titles = vec![
            "Inventory".to_owned(),
            "Tomato".to_owned(),
            "Statistics".to_owned(),
        ];
        NavitabModel { titles, select: 0 }
    }
    pub fn next(&mut self) {
        self.select = (self.select + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.select > 0 {
            self.select -= 1;
        } else {
            self.select = self.titles.len() - 1;
        }
    }

    pub fn tab_type(&self) -> TabType {
        match self.select {
            0 => TabType::Inventory,
            1 => TabType::Tomato,
            2 => TabType::Statistics,
            _ => TabType::Inventory,
        }
    }
}
