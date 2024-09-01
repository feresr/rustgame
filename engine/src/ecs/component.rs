use super::Updateable;
use imgui::Ui;
use std::{
    any::{type_name, Any, TypeId},
    cell::{RefCell, RefMut},
    collections::HashMap,
};

pub trait Component {
    // Default capacity for the component storage.
    // This is used to allocate the initial size of the component storage
    // and is used to reserve space for the components.
    // This is used to optimize the allocation of the component storage.
    // Commonly occuring Components (Position, Velocity, etc.) should increase this number.
    const CAPACITY: usize = 16;
}

#[derive(Debug)]
pub struct ComponentWrapper<T: Component> {
    pub entity_id: u32,
    pub component: RefCell<T>,
}

pub struct ComponentStorage<T: Component> {
    pub data: Vec<ComponentWrapper<T>>, // Store components contiguously
    pub entity_map: HashMap<u32, usize>, // Map entity ID to component index
    pub type_id: TypeId,
}

impl<T: Component + 'static> ComponentStorage<T> {
    pub fn remove_component(&mut self, entity_id: u32) -> Option<T> {
        if let Some(index) = self.entity_map.remove(&entity_id) {
            let last_entity_id = self.data.iter().last().unwrap().entity_id;
            let value = self.data.swap_remove(index);
            if entity_id != last_entity_id {
                self.entity_map.insert(last_entity_id, index);
            }
            return Some(value.component.into_inner());
        }
        return None;
    }
}

impl<T: Component + 'static> Updateable for ComponentStorage<T> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove_component(&mut self, entity_id: u32) {
        if let Some(index) = self.entity_map.remove(&entity_id) {
            let last_entity_id = self.data.iter().last().unwrap().entity_id;
            self.data.swap_remove(index);
            if entity_id != last_entity_id {
                self.entity_map.insert(last_entity_id, index);
            }
        }
    }

    fn add_component(&mut self, entity_id: u32, component: Box<dyn Any>) {
        self.entity_map.insert(entity_id, self.data.len());
        let component = component.downcast::<T>().unwrap();
        self.data.push(ComponentWrapper {
            entity_id,
            component: RefCell::new(*component),
        });
    }

    fn debug(&self, ui: &Ui) {
        ui.text(format!(
            "{} # {:?}",
            type_name::<T>(),
            self.data.iter().count()
        ));
    }
}
impl<T: Component> ComponentStorage<T> {
    // Create a new empty ComponentStorage
    pub fn new(type_id: TypeId, capacity: usize) -> Self {
        ComponentStorage {
            data: Vec::with_capacity(capacity),
            entity_map: HashMap::with_capacity(capacity),
            type_id,
        }
    }
    // fn remove(&mut self, entity_id: u32) {
    //     let index = self.entity_map.get(&entity_id).unwrap();
    //     self.data.remove(*index);
    // }

    // Add a component to the storage and associate it with an entity
    pub fn add_component(&mut self, entity_id: u32, component: T) {
        self.entity_map.insert(entity_id, self.data.len());
        self.data.push(ComponentWrapper {
            entity_id,
            component: RefCell::new(component),
        });
    }

    pub fn find_component(&self, entity_id: u32) -> Option<RefMut<'_, T>> {
        if let Some(index) = self.entity_map.get(&entity_id) {
            let wrapper = self.data.get(*index);
            return wrapper.map(|wrapper| wrapper.component.borrow_mut());
        }
        return None;
    }
}
