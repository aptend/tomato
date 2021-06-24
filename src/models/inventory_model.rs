use tui::style::Color;

pub struct InventoryModel {
    pub inventory_selected: Option<usize>,
    pub task_selected: Vec<Option<usize>>,
    pub inventory_list: Vec<Inventory>,
    pub tasks_list: Vec<Vec<Task>>,
}

impl InventoryModel {
    
    pub fn get_task_location(&self) -> Option<(usize, usize)> {
        if let Some(iidx) = self.inventory_selected {
            if let Some(tidx) = self.task_selected[iidx] {
                return Some((iidx, tidx));
            }
        }
        None
    }

    fn next<T>(&self, selected: Option<usize>, list: &[T]) -> Option<usize> {
        if list.is_empty() {
            return None;
        }
        match selected {
            Some(idx) => Some((idx + 1) % list.len()),
            None => Some(0),
        }
    }

    fn previous<T>(&self, selected: Option<usize>, list: &[T]) -> Option<usize> {
        if list.is_empty() {
            return None;
        }
        match selected {
            Some(idx) => Some(if idx == 0 { list.len() - 1 } else { idx - 1 }),
            None => Some(list.len() - 1),
        }
    }

    pub fn next_inventory(&mut self) {
        self.inventory_selected = self.next(self.inventory_selected, &self.inventory_list);
    }

    pub fn next_task(&mut self) {
        if let Some(idx) = self.inventory_selected {
            self.task_selected[idx] = self.next(self.task_selected[idx], &self.tasks_list[idx]);
        }
    }

    pub fn previous_inventory(&mut self) {
        self.inventory_selected = self.previous(self.inventory_selected, &self.inventory_list);
    }

    pub fn previous_task(&mut self) {
        if let Some(idx) = self.inventory_selected {
            self.task_selected[idx] = self.previous(self.task_selected[idx], &self.tasks_list[idx]);
        }
    }
}

pub struct Inventory {
    pub name: String,
    pub color: Color,
}

pub struct Task {
    pub name: String,
    pub tomato_minutes: usize,
    pub crate_date: String,
    pub notes: String,
}
