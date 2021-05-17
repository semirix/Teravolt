use crate::types::*;
use std::any::{Any, TypeId};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard};

pub trait ResourceTrait: Any + Send + Sync + Sized + Default {}
impl<T> ResourceTrait for T where T: Any + Send + Sync + Sized + Default {}

/// A universal resource type for transmitting data inside Teravolt.
#[derive(Debug, Clone)]
struct InternalResource(Arc<RwLock<Box<dyn Any + Send + Sync>>>, TypeId);

impl InternalResource {
    /// Creates a new Resource.
    fn new<T: ResourceTrait>(data: T) -> Self {
        Self(Arc::new(RwLock::new(Box::new(data))), TypeId::of::<T>())
    }

    /// Immutably read resource. This will panic if you don't use the correct
    /// type.
    fn as_typed<T: ResourceTrait>(&self) -> Resource<T> {
        Resource(self.0.clone(), self.1, PhantomData::<T>)
    }
}

/// A universal resource type for transmitting data inside Teravolt.
#[derive(Debug, Clone)]
pub struct Resource<T>(
    Arc<RwLock<Box<dyn Any + Send + Sync>>>,
    TypeId,
    PhantomData<T>,
);

impl<T> Resource<T>
where
    T: ResourceTrait,
{
    /// Get an immutable reference to the resource.
    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        RwLockReadGuard::map(self.0.read().await, |data| {
            // Theoretically, this should never panic
            data.downcast_ref::<T>().unwrap()
        })
    }

    /// Get a mutable reference to the resource.
    pub async fn write(&self) -> RwLockMappedWriteGuard<'_, T> {
        RwLockWriteGuard::map(self.0.write().await, |data| {
            // Theoretically, this should never panic
            data.downcast_mut::<T>().unwrap()
        })
    }
}

/// A global storage container that is passed down to connection tasks.
#[derive(Debug, Clone)]
pub struct Storage {
    map: Arc<AsyncMap<TypeId, InternalResource>>,
}

impl Storage {
    /// Create a new Storage object.
    pub fn new() -> Self {
        Self {
            map: Arc::new(async_map()),
        }
    }

    /// Get a handle on a Resource within storage.
    pub async fn handle<T: ResourceTrait>(&self) -> Resource<T> {
        let exists = if let Some(_) = self.map.read().await.get(&TypeId::of::<T>()) {
            true
        } else {
            false
        };

        if exists {
            self.map
                .read()
                .await
                .get(&TypeId::of::<T>())
                .unwrap()
                .as_typed()
        } else {
            let resource = InternalResource::new(T::default());
            {
                self.map
                    .write()
                    .await
                    .insert(TypeId::of::<T>(), resource.clone());
            }
            resource.as_typed()
        }
    }

    /// Clear out a resource from storage.
    pub async fn clear<T: ResourceTrait>(&self) {
        if let Some(resource) = self.map.write().await.get(&TypeId::of::<T>()) {
            let data = resource.as_typed();
            *data.write().await = T::default();
        }
    }
}
