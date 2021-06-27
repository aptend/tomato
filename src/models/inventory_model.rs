use crate::db::{DbUtils, Inventory, Task};
pub struct InventoryModel {
    pub inventory_selected: Option<usize>,
    pub task_selected: Vec<Option<usize>>,
    pub inventory_list: Vec<Inventory>,
    pub tasks_list: Vec<Vec<Task>>,
}

impl InventoryModel {
    pub fn new() -> Self {
        let inventory_list = DbUtils::all_inventory();
        let tasks_list = DbUtils::all_task_groupby(&inventory_list);
        assert_eq!(inventory_list.len(), tasks_list.len());
        InventoryModel {
            task_selected: vec![None; tasks_list.len()],
            inventory_selected: None,
            inventory_list,
            tasks_list,
        }
    }

    pub fn push_new_inventory(&mut self, inv: Inventory) {
        self.inventory_list.push(inv);
        self.task_selected.push(None);
        self.tasks_list.push(Vec::new());
    }

    pub fn push_new_task(&mut self, task: Task) {
        for (idx, inv) in self.inventory_list.iter().enumerate() {
            if inv.id == task.inventory_id {
                self.tasks_list[idx].push(task);
                return;
            }
        }
    }

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
