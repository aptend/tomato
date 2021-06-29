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

    fn inventory_idx_by_id(&self, id: i32) -> Option<usize> {
        self.inventory_list
            .iter()
            .enumerate()
            .find(|(_, inv)| inv.id == id)
            .map(|(idx, _)| idx)
    }

    pub fn push_new_task(&mut self, task: Task) {
        if let Some(idx) = self.inventory_idx_by_id(task.inventory_id) {
            self.tasks_list[idx].push(task);
        }
    }

    pub fn delete_task(&mut self, inventory_id: i32, task_id: i32) {
        if let Some(idx) = self.inventory_idx_by_id(inventory_id) {
            self.tasks_list[idx].retain(|t| t.id != task_id);
            self.next_task();
        }
    }

    pub fn delete_inventory(&mut self, inventory_id: i32) {
        if let Some(idx) = self.inventory_idx_by_id(inventory_id) {
            self.inventory_list.remove(idx);
            self.task_selected.remove(idx);
            self.tasks_list.remove(idx);
            self.next_inventory();
        }
    }

    pub fn get_task_location(&self) -> Option<(usize, usize)> {
        self.inventory_selected
            .and_then(|iidx| self.task_selected[iidx].map(|tidx| (iidx, tidx)))
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
