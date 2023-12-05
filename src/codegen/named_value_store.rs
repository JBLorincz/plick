use std::{cell::RefCell, collections::HashMap, fmt::Debug};

use inkwell::context::Context;

use super::named_value::NamedValue;

pub trait NamedValueStore<'ctx> {
    fn clear(&self);
    fn insert(&self, named_value: NamedValue<'ctx>);
    fn try_remove(&self, name: &str) -> Result<(), String>;
    fn try_get(&self, name: &str) -> Option<NamedValue<'ctx>>;
}
impl<'ctx> Debug for dyn NamedValueStore<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "test!")
    }
}

#[derive(Debug)]
pub struct NamedValueHashmapStore<'ctx> {
    pub named_values: RefCell<HashMap<String, NamedValue<'ctx>>>,
}

impl<'ctx> NamedValueHashmapStore<'ctx> {
    pub fn new() -> Self {
        let named_values: RefCell<HashMap<String, NamedValue<'_>>> = RefCell::new(HashMap::new());
        NamedValueHashmapStore { named_values }
    }
}

impl<'ctx> NamedValueStore<'ctx> for NamedValueHashmapStore<'ctx> {
    fn clear(&self) {
        self.named_values.borrow_mut().clear();
    }

    fn insert(&self, named_value: NamedValue<'ctx>) {
        self.named_values
            .borrow_mut()
            .insert(named_value.name.clone(), named_value);
    }

    fn try_remove(&self, name: &str) -> Result<(), String> {
        self.named_values.borrow_mut().remove_entry(name);
        Ok(())
    }

    fn try_get(&self, name: &str) -> Option<NamedValue<'ctx>> {
        Some(self.named_values.borrow().get(name)?.clone())
    }
}
