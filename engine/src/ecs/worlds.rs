use std::{
    any::{Any, TypeId},
    cell::RefMut,
    collections::HashMap,
};

use rand::Rng;

use super::{
    Component, ComponentStorage, ComponentWrapper, Diff, Entity, IEntity, Updateable, WorldOp,
};

pub struct RenderWorld<'a> {
    pub components: &'a HashMap<TypeId, Box<dyn Updateable>>,
    pub resources: &'a mut HashMap<TypeId, Box<dyn Any>>,
}

impl RenderWorld<'_> {
    pub fn get_resource<T: 'static>(&mut self) -> &mut T {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .as_mut()
            .downcast_mut::<T>()
            .unwrap()
    }
}

pub struct UpdateWorld<'a> {
    // nont mut ref to current world components
    pub components: &'a HashMap<TypeId, Box<dyn Updateable>>,
    pub resources: &'a mut HashMap<TypeId, Box<dyn Any>>,
    pub diffs: &'a mut Vec<Diff>, // HashMap for component storages by TypeId
}

impl UpdateWorld<'_> {
    pub fn get_resource<T: 'static>(&mut self) -> &mut T {
        self.resources
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .as_mut()
            .downcast_mut::<T>()
            .unwrap()
    }
}

impl<'a> WorldOp for RenderWorld<'a> {
    fn add_entity(&mut self) -> Entity<'_, RenderWorld<'a>> {
        panic!("Cannot modify the world during render");
    }

    fn remove_entity(&mut self, _entity: u32) {
        panic!("Cannot modify the world during render");
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

    fn remove_component<T: Component + 'static>(&mut self, entity: u32) {
        panic!("Cannot modify the world during render");
    }

    fn add_component<T: Component + 'static>(&mut self, entity: &IEntity, component: T) {
        panic!("Cannot modify the world during render");
    }

    fn find_first<T: Component + 'static>(&mut self) -> Option<Entity<'_, impl WorldOp>> {
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

    fn find_all<T: Component + 'static>(
        &self,
    ) -> Box<dyn Iterator<Item = &ComponentWrapper<T>> + '_> {
        Box::new(std::iter::empty())
    }
}
impl<'a> WorldOp for UpdateWorld<'a> {
    fn add_entity(&mut self) -> Entity<'_, impl WorldOp> {
        let rng: u32 = rand::thread_rng().r#gen();
        self.diffs.push(Diff::AddEntity { id: rng });
        return Entity {
            id: rng,
            world: self,
        };
    }

    fn remove_entity(&mut self, entity: u32) {
        self.diffs.push(Diff::RemoveEntity { id: entity });
    }

    fn find_component<T: Component + 'static>(&self, entity: u32) -> Option<RefMut<'_, T>> {
        if let Some(storage) = self.components.get(&TypeId::of::<T>()) {
            if let Some(storage) = storage.as_any().downcast_ref::<ComponentStorage<T>>() {
                return storage.find_component(entity);
            }
        }
        return None;
    }

    fn remove_component<T: Component + 'static>(&mut self, entity: u32) {
        self.diffs.push(Diff::RemoveComponent {
            component_type: TypeId::of::<T>(),
            entity,
        });
    }

    fn add_component<T: Component + 'static>(&mut self, entity: &IEntity, component: T) {
        let type_id = TypeId::of::<T>();
        self.diffs.push(Diff::AddComponent {
            component_type: type_id,
            entity: entity.id,
            component: Box::new(component),
        });
    }

    fn find_first<T: Component + 'static>(&mut self) -> Option<Entity<'_, impl WorldOp>> {
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

    // TODO: find a way to return a impl iterator (not dyn)
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
                // return storage.data.iter();
                return Box::new(storage.data.iter());
            }
            None => return Box::new(std::iter::empty()),
        }
    }
}
