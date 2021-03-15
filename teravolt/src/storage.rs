use crate::types::*;
use std::any::{Any, TypeId};
use std::sync::Arc;
use tokio::sync::RwLock;

pub trait ResourceTrait: Any + Send + Sync + Sized + Default {}
impl<T> ResourceTrait for T where T: Any + Send + Sync + Sized + Default {}

/// A universal resource type for transmitting data inside Teravolt.
#[derive(Debug, Clone)]
pub struct Resource(Arc<RwLock<Box<dyn Any + Send + Sync>>>, TypeId);

impl Resource {
    /// Creates a new Resource.
    fn new<T: Any + Send + Sync>(data: T) -> Self {
        Self(Arc::new(RwLock::new(Box::new(data))), TypeId::of::<T>())
    }

    /// Immutably read resource from closure.
    async fn read<T: Any + Send + Sync>(&self, closure: fn(&T)) {
        if let Some(data) = self.0.read().await.downcast_ref::<T>() {
            closure(data);
        }
    }

    /// Mutably read resource from closure.
    async fn write<T: Any + Send + Sync>(&self, closure: fn(&mut T)) {
        if let Some(data) = self.0.write().await.downcast_mut::<T>() {
            closure(data);
        }
    }
}

/// A global storage container that is passed down to connection tasks.
#[derive(Debug, Clone)]
pub struct Storage {
    map: Arc<AsyncMap<TypeId, Resource>>,
}

impl Storage {
    /// Create a new Storage object.
    pub fn new() -> Self {
        Self {
            map: Arc::new(async_map()),
        }
    }

    /// Immutably read resource in storage from closure.
    pub async fn read<T: ResourceTrait>(&self, closure: fn(&T)) {
        if !{
            if let Some(resource) = self.map.read().await.get(&TypeId::of::<T>()) {
                resource.read(closure).await;
                true
            } else {
                false
            }
        } {
            let resource = Resource::new(T::default());
            self.map
                .write()
                .await
                .insert(TypeId::of::<T>(), resource.clone());
            resource.read(closure).await;
        }
    }

    /// Mutably read resource in storage from closure.
    pub async fn write<T: ResourceTrait>(&self, closure: fn(&mut T)) {
        let mut data = self.map.write().await;

        if let Some(resource) = data.get(&TypeId::of::<T>()) {
            resource.write(closure).await;
        } else {
            let resource = Resource::new(T::default());
            data.insert(TypeId::of::<T>(), resource.clone());
            resource.write(closure).await;
        }
    }

    /// Get a handle on a Resource within storage.
    pub async fn handle<T: ResourceTrait>(&self) -> Resource {
        if let Some(resource) = self.map.read().await.get(&TypeId::of::<T>()) {
            resource.clone()
        } else {
            let resource = Resource::new(T::default());
            self.map
                .write()
                .await
                .insert(TypeId::of::<T>(), resource.clone());
            resource
        }
    }
}
