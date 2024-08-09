use std::any::{type_name, Any, TypeId};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::Debug;

use imgui::Ui;
use rand::Rng;

use crate::graphics::batch::Batch;

// Internal entity (no world reference - self ref not allowed in Rust)
// TODO: should this be pub?
#[derive(Clone, Copy, Debug)]
pub struct IEntity {
    id: u32,
}

pub struct Entity<'a, T: WorldOp<T>> {
    id: u32,
    world: &'a mut T,
}

#[derive(Debug)]
pub struct ComponentWrapper<T: Component> {
    pub entity_id: u32,
    pub component: RefCell<T>,
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

pub trait Component {
    fn update<'a>(&mut self, world: &'a mut UpdateWorld<'_>, entity: u32);
    fn render<'a>(&mut self, world: &'a mut RenderWorld<'_>, batch: &mut Batch, entity: u32);
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

// Define a ComponentStorage struct that stores components in contiguous memory
struct ComponentStorage<T: Component> {
    data: Vec<ComponentWrapper<T>>,  // Store components contiguously
    entity_map: HashMap<u32, usize>, // Map entity ID to component index
}

impl<T: Component + 'static> Updateable for ComponentStorage<T> {
    // Iterate over components to update them
    fn update_all(&self, world: &mut UpdateWorld<'_>) {
        for wrapper in self.data.iter() {
            wrapper
                .component
                .borrow_mut()
                .update(world, wrapper.entity_id);
        }
    }
    // Iterate over components to render them
    fn render_all(&self, world: &mut RenderWorld<'_>, batch: &mut Batch) {
        for wrapper in self.data.iter() {
            wrapper
                .component
                .borrow_mut()
                .render(world, batch, wrapper.entity_id);
        }
    }
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
            if (entity_id != last_entity_id) {
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
    fn new() -> Self {
        ComponentStorage {
            data: Vec::with_capacity(100),
            entity_map: HashMap::new(),
        }
    }
    // fn remove(&mut self, entity_id: u32) {
    //     let index = self.entity_map.get(&entity_id).unwrap();
    //     self.data.remove(*index);
    // }

    // Add a component to the storage and associate it with an entity
    fn add_component(&mut self, entity_id: u32, component: T) {
        self.entity_map.insert(entity_id, self.data.len());
        self.data.push(ComponentWrapper {
            entity_id,
            component: RefCell::new(component),
        });
    }

    fn find_component(&self, entity_id: u32) -> Option<RefMut<'_, T>> {
        if let Some(index) = self.entity_map.get(&entity_id) {
            let wrapper = self.data.get(*index);
            return wrapper.map(|wrapper| wrapper.component.borrow_mut());
        }
        return None;
    }
}
//
enum Diff {
    AddEntity {
        id: u32,
    },
    RemoveEntity {
        id: u32,
    },
    RemoveComponent {
        component_type: TypeId,
        entity: u32,
    },
    AddComponent {
        component_type: TypeId,
        entity: u32,
        component: Box<dyn Any>,
    },
}
pub struct RenderWorld<'a> {
    components: &'a HashMap<TypeId, Box<dyn Updateable>>,
    resources: &'a mut HashMap<TypeId, Box<dyn Any>>,
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
    components: &'a HashMap<TypeId, Box<dyn Updateable>>,
    resources: &'a mut HashMap<TypeId, Box<dyn Any>>,
    diffs: &'a mut Vec<Diff>, // HashMap for component storages by TypeId
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

impl<'a> WorldOp<RenderWorld<'a>> for RenderWorld<'a> {
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

    fn find_first<T: Component + 'static>(&mut self) -> Option<Entity<'_, RenderWorld<'a>>> {
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
impl<'a> WorldOp<UpdateWorld<'a>> for UpdateWorld<'a> {
    fn add_entity(&mut self) -> Entity<'_, UpdateWorld<'a>> {
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

    fn find_first<T: Component + 'static>(&mut self) -> Option<Entity<'_, UpdateWorld<'a>>> {
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

pub trait WorldOp<W: WorldOp<W>> {
    fn add_entity<'a>(&'a mut self) -> Entity<'_, W>;
    fn remove_entity<'a>(&'a mut self, entity: u32);

    fn add_component<T: Component + 'static>(&mut self, entity: &IEntity, component: T);
    fn remove_component<T: Component + 'static>(&mut self, entity: u32);
    fn find_component<T: Component + 'static>(&self, entity: u32) -> Option<RefMut<'_, T>>;

    fn find_first<'a, T: Component + 'static>(&'a mut self) -> Option<Entity<'_, W>>;
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

    // Register a new component type with an empty storage
    fn register_component<T: Component + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        self.components
            .insert(type_id, Box::new(ComponentStorage::<T>::new()));
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

impl WorldOp<World> for World {
    // Add a new entity to the world and return it
    fn add_entity(&mut self) -> Entity<'_, World> {
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

    fn find_first<T: Component + 'static>(&mut self) -> Option<Entity<'_, World>> {
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

impl<W: WorldOp<W>> Entity<'_, W> {
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
