pub mod component;
mod worlds;

use std::any::{Any, TypeId};
use std::cell::RefMut;
use std::collections::HashMap;
use std::fmt::Debug;

pub use component::{Component, ComponentStorage, ComponentWrapper, Diff};
use imgui::Ui;
use rand::Rng;
use worlds::{RenderWorld, UpdateWorld};

use crate::graphics::batch::Batch;

// Internal entity (no world reference - self ref not allowed in Rust)
// TODO: should this be pub?
#[derive(Clone, Copy, Debug)]
pub struct IEntity {
    id: u32,
}

pub struct Entity<'a, T: WorldOp> {
    pub id: u32,
    pub world: &'a mut T,
}

// World struct that manages entities and component storages
pub struct World {
    // todo: make entities private again, maybe even remove?????
    entities: Vec<IEntity>, // Vec to store entities
    pub entity_count: u32,
    components: HashMap<TypeId, Box<dyn Updateable>>, // HashMap for component storages by TypeId
    resources: HashMap<TypeId, Box<dyn Any>>,
    changes: Vec<Diff>,
}

trait Updateable {
    fn update_all(&self, world: &mut UpdateWorld<'_>);
    fn render_all(&self, world: &mut RenderWorld<'_>, batch: &mut Batch);

    // Move this to where add_component is?
    fn remove_component(&mut self, entity_id: u32);
    fn add_component(&mut self, entity_id: u32, component: Box<dyn Any>);

    // I have no idea how / why this works
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn debug(&self, ui: &Ui);
}

pub trait WorldOp {
    fn add_entity<'a>(&'a mut self) -> Entity<'_, impl WorldOp>;
    fn remove_entity<'a>(&'a mut self, entity: u32);

    fn add_component<T: Component + 'static>(&mut self, entity: &IEntity, component: T);
    fn remove_component<T: Component + 'static>(&mut self, entity: u32);
    fn find_component<'a, T: Component + 'static>(&'a self, entity: u32) -> Option<RefMut<'a, T>>;

    fn find_first<'a, T: Component + 'static>(&'a mut self) -> Option<Entity<'_, impl WorldOp>>;
    fn find_all<T: Component + 'static>(
        &self,
    ) -> Box<dyn Iterator<Item = &ComponentWrapper<T>> + '_>;
}

impl World {
    // Create a new World instance
    pub fn new() -> Self {
        World {
            entities: Vec::new(),
            entity_count: 0,
            components: HashMap::new(),
            changes: Vec::new(),
            resources: HashMap::new(),
        }
    }

    pub fn add_resource<T: Any + 'static>(&mut self, resource: T) {
        self.resources
            .insert(resource.type_id(), Box::new(resource));
    }
    pub fn unassign<T: Component + 'static>(&mut self) -> T {
        todo!()
    }

    // Register a new component type with an empty storage
    fn register_component<T: Component + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.components
            .insert(type_id, Box::new(ComponentStorage::<T>::new()));
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

    pub fn update(&mut self) {
        let mut diff = UpdateWorld {
            components: &self.components,
            diffs: &mut self.changes,
            resources: &mut self.resources,
        };
        for (_, updatable) in self.components.iter() {
            updatable.update_all(&mut diff);
        }
        for diff in self.changes.drain(..) {
            match diff {
                Diff::RemoveComponent {
                    component_type,
                    entity,
                } => {
                    let type_id = component_type;
                    let components_optional = self.components.get_mut(&type_id);
                    if let Some(updateable) = components_optional {
                        updateable.remove_component(entity);
                    }
                }
                Diff::AddComponent {
                    component_type,
                    entity,
                    component,
                } => {
                    // TODO unwrap is dangeourse here
                    // self.add_component(&IEntity{id: entity}, component):
                    let c = self.components.get_mut(&component_type).unwrap();
                    c.add_component(entity, component);
                }
                Diff::AddEntity { id } => {
                    self.entity_count = self.entity_count + 1;
                }
                Diff::RemoveEntity { id } => {
                    // can't call this without double borrow
                    // self.remove_component(entity);
                    self.entity_count = self.entity_count - 1;
                    for (_, updatable) in self.components.iter_mut() {
                        updatable.remove_component(id);
                    }
                    self.entities.retain(|e| e.id != id);
                }
            }
        }
    }

    pub fn render(&self, batch: &mut Batch) {
        let mut diff = RenderWorld {
            components: &self.components,
            resources: &mut HashMap::new(),
        };
        for (_, updatable) in self.components.iter() {
            updatable.render_all(&mut diff, batch);
        }
        // TODO: update world with diff data?
        // self.changes.clear();
    }

    pub fn debug(&self, imgui: &Ui) {
        imgui.text(format!("ENTITIES # {}", self.entity_count));
        for (_, update) in self.components.iter() {
            update.debug(&imgui);
        }
    }
}

impl WorldOp for World {
    // Add a new entity to the world and return it
    fn add_entity(&mut self) -> Entity<'_, impl WorldOp> {
        // let id = self.entities.len() as u32;
        self.entity_count = self.entity_count + 1;
        let rng: u32 = rand::thread_rng().r#gen();
        self.entities.push(IEntity { id: rng });
        return Entity {
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

    fn find_component<T: Component + 'static>(&self, entity: u32) -> Option<RefMut<'_, T>> {
        // TODo imrp
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

    fn find_first<'a, T: Component + 'static>(&'a mut self) -> Option<Entity<'a, World>> {
        let type_id = TypeId::of::<T>();
        if let None = self.components.get(&type_id) {
            return None;
        }
        if let Some(storage) = self.components.get_mut(&type_id) {
            if let Some(storage) = storage.as_any_mut().downcast_mut::<ComponentStorage<T>>() {
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

    fn find_all<T: Component + 'static>(
        &self,
    ) -> Box<dyn Iterator<Item = &ComponentWrapper<T>> + '_> {
        Box::new(std::iter::empty())
    }
}

impl<'a> Entity<'a, World> {
    pub fn extract_component<T: Component + 'static>(&mut self) -> Option<T> {
        return self.world.extract_component::<T>(self.id);
    }
}

impl<'a, W: WorldOp> Entity<'a, W> {
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
