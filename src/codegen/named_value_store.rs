use std::{cell::RefCell, collections::HashMap};

use super::codegen::NamedValue;

pub trait NamedValueStore
{
    fn clear(&self);
    fn insert(&self, named_value: NamedValue);
    fn try_remove(&self,name: String) -> Result<(),String>;
}

pub struct NamedValueHashmapStore<'ctx>
{
    pub named_values: RefCell<HashMap<String,NamedValue<'ctx>>>,
}

impl<'ctx> NamedValueHashmapStore<'ctx>
{
    pub fn new() -> Self
    {

        let named_values: RefCell<HashMap<String,NamedValue<'ctx>>> = RefCell::new(HashMap::new());
        NamedValueHashmapStore { named_values }
    }
}

impl<'ctx> NamedValueStore for NamedValueHashmapStore<'ctx>
{
    fn clear(&self)
    {
        self.named_values.borrow_mut().clear();
    }

    fn insert(&self, named_value: NamedValue)
    {
        self.named_values.borrow_mut().insert(named_value.name, named_value);
    }

    fn try_remove(&self,name: String) -> Result<(),String>
    {
        self.named_values.borrow_mut().remove_entry(&name);
        Ok(())
    }

}
