pub mod component;

use std::any::{Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

pub use component::{Component, ComponentStorage, ComponentWrapper};
use imgui::Ui;
use rand::Rng;

pub type Resource = Rc<RefCell<Box<dyn Any>>>;
// Internal entity (no world reference - self ref not allowed in Rust)
// TODO: should this be pub?
#[derive(Clone, Copy, Debug)]
pub struct IEntity {
    id: u32,
}

pub struct Entity<'a> {
    pub id: u32,
    pub world: &'a World,
}

pub struct EntityMut<'a> {
    pub id: u32,
    pub world: &'a mut World,
}

// World struct that manages entities and component storages
pub struct World {
    pub entity_count: u32,
    // todo: make entities private again, maybe even remove?????
    entities: Vec<IEntity>,
    components: HashMap<TypeId, Box<dyn Updateable>>,
    resources: HashMap<TypeId, Resource>,
}

trait Updateable {
    // I have no idea how / why this works
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn debug(&self, ui: &Ui);
    fn remove_component(&mut self, entity_id: u32);
}

pub trait WorldOp {
    fn add_entity<'a>(&'a mut self) -> EntityMut<'_>;
    fn remove_entity<'a>(&'a mut self, entity: u32);

    fn add_component<T: Component + 'static>(&mut self, entity: &IEntity, component: T);
    fn remove_component<T: Component + 'static>(&mut self, entity: u32);
    fn find_component<'a, T: Component + 'static>(&'a self, entity: u32) -> Option<RefMut<'a, T>>;

    fn entity(&self, entity: u32) -> Entity<'_>;
    fn entity_mut(&mut self, entity: u32) -> EntityMut<'_>;

    fn first<'a, T: Component + 'static>(&'a self) -> Option<Entity<'_>>;
    fn all_with<T: Component + 'static>(&self) -> Box<dyn Iterator<Item = Entity<'_>> + '_>;
    fn find_all<T: Component + 'static>(
        &self,
    ) -> Box<dyn Iterator<Item = &ComponentWrapper<T>> + '_>;
}

impl World {
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            entity_count: 0,
            components: HashMap::with_capacity(64),
            resources: HashMap::with_capacity(8),
        }
    }

    pub fn add_resource<T: Any + 'static>(&mut self, resource: T) {
        self.resources.insert(
            resource.type_id(),
            Rc::new(RefCell::new(Box::new(resource))),
        );
    }
    pub fn unassign<T: Component + 'static>(&mut self) -> T {
        todo!()
    }

    // Register a new component type with an empty storage
    fn register_component<T: Component + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.components.insert(
            type_id,
            Box::new(ComponentStorage::<T>::new(type_id, T::CAPACITY)),
        );
    }

    pub fn extract_component<T: Component + 'static>(&mut self, entity_id: u32) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get_mut(&type_id) {
            if let Some(storage) = storage.as_any_mut().downcast_mut::<ComponentStorage<T>>() {
                return storage.remove_component(entity_id);
            }
        }
        return None;
    }

    pub fn debug(&self, ui: &Ui) {
        for (_, storage) in self.components.iter() {
            storage.debug(ui);
        }
    }
}

impl WorldOp for World {
    // Add a new entity to the world and return it
    fn add_entity(&mut self) -> EntityMut<'_> {
        // let id = self.entities.len() as u32;
        self.entity_count = self.entity_count + 1;
        let rng: u32 = rand::thread_rng().r#gen();
        self.entities.push(IEntity { id: rng });
        return EntityMut {
            id: rng,
            world: self,
        };
    }
    fn remove_entity<'a>(&'a mut self, entity: u32) {
        self.entity_count -= 1;
        for (_, updatable) in self.components.iter_mut() {
            updatable.remove_component(entity);
        }
        self.entities.retain(|e| e.id != entity);
    }

    fn entity(&self, entity: u32) -> Entity<'_> {
        return Entity {
            id: entity,
            world: self,
        };
    }

    fn entity_mut(&mut self, entity: u32) -> EntityMut<'_> {
        return EntityMut {
            id: entity,
            world: self,
        };
    }

    fn find_component<T: Component + 'static>(&self, entity: u32) -> Option<RefMut<'_, T>> {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get(&type_id) {
            if let Some(storage) = storage.as_any().downcast_ref::<ComponentStorage<T>>() {
                return storage.find_component(entity);
            }
        }
        return None;
    }

    // Add a component to the specified entity's component storage
    fn remove_component<T: Component + 'static>(&mut self, entity: u32) {
        let type_id = TypeId::of::<T>();
        if let Some(storage) = self.components.get_mut(&type_id) {
            if let Some(storage) = storage.as_any_mut().downcast_mut::<ComponentStorage<T>>() {
                storage.remove_component(entity);
            }
        }
    }
    // Add a component to the specified entity's component storage
    fn add_component<T: Component + 'static>(&mut self, entity: &IEntity, component: T) {
        let type_id = TypeId::of::<T>();
        if let None = self.components.get(&type_id) {
            self.register_component::<T>();
        }
        if let Some(storage) = self.components.get_mut(&type_id) {
            if let Some(storage) = storage.as_any_mut().downcast_mut::<ComponentStorage<T>>() {
                storage.add_component(entity.id, component)
            }
        }
    }

    fn first<'a, T: Component + 'static>(&'a self) -> Option<Entity<'a>> {
        let type_id = TypeId::of::<T>();
        if let None = self.components.get(&type_id) {
            return None;
        }
        if let Some(storage) = self.components.get(&type_id) {
            if let Some(storage) = storage.as_any().downcast_ref::<ComponentStorage<T>>() {
                for wrapper in storage.data.iter() {
                    return Some(Entity {
                        id: wrapper.entity_id,
                        world: self,
                    });
                }
            }
        }
        return None;
    }

    // TODO: first with
    fn all_with<T: Component + 'static>(&self) -> Box<dyn Iterator<Item = Entity<'_>> + '_> {
        let type_id = TypeId::of::<T>();
        match self.components.get(&type_id) {
            Some(storage) => {
                let storage = storage
                    .as_any()
                    .downcast_ref::<ComponentStorage<T>>()
                    .unwrap();
                return Box::new(storage.data.iter().map(|a| Entity {
                    id: a.entity_id,
                    world: self,
                }));
            }
            None => return Box::new(std::iter::empty()),
        }
    }

    fn find_all<T: Component + 'static>(
        &self,
    ) -> Box<dyn Iterator<Item = &ComponentWrapper<T>> + '_> {
        let type_id = TypeId::of::<T>();
        match self.components.get(&type_id) {
            Some(storage) => {
                let storage = storage
                    .as_any()
                    .downcast_ref::<ComponentStorage<T>>()
                    .unwrap();
                return Box::new(storage.data.iter());
            }
            None => return Box::new(std::iter::empty()),
        }
    }
}

impl<'a> EntityMut<'a> {
    pub fn extract_component<T: Component + 'static>(&mut self) -> Option<T> {
        return self.world.extract_component::<T>(self.id);
    }
}

impl<'a> Entity<'a> {
    pub fn get<T: Component + 'static>(&self) -> RefMut<'_, T> {
        return self
            .world
            .find_component::<T>(self.id)
            .expect("not present");
    }
    pub fn has<T: Component + 'static>(&self) -> Option<RefMut<'_, T>> {
        return self.world.find_component::<T>(self.id);
    }
}

impl<'a> EntityMut<'a> {
    // Adds a component to this entity
    pub fn assign<T: Component + 'static>(&mut self, component: T) {
        let entity = IEntity { id: self.id };
        self.world.add_component(&entity, component)
    }
    // TODO: consider returning the result
    pub fn unassign<T: Component + 'static>(&mut self) {
        self.world.remove_component::<T>(self.id)
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<RefMut<'_, T>> {
        return self.world.find_component::<T>(self.id);
    }
}
