use crate::error::error;
use crate::Result;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use tokio::sync::{RwLock, RwLockMappedWriteGuard, RwLockReadGuard, RwLockWriteGuard};

pub trait ResourceTrait: Any + Send + Sync + Sized + Default {}
impl<T> ResourceTrait for T where T: Any + Send + Sync + Sized + Default {}

/// A universal resource type for transmitting data inside Teravolt.
#[derive(Debug, Clone)]
struct InternalResource(Arc<RwLock<Box<dyn Any + Send + Sync>>>, TypeId);

impl InternalResource {
    /// Creates a new Resource.
    #[tracing::instrument]
    fn new<T: ResourceTrait + Debug>(data: T) -> Self {
        Self(Arc::new(RwLock::new(Box::new(data))), TypeId::of::<T>())
    }

    /// Immutably read resource. This will panic if you don't use the correct
    /// type.
    #[tracing::instrument]
    fn as_typed<T: ResourceTrait>(&self) -> Resource<T> {
        Resource(self.0.clone(), PhantomData::<T>)
    }
}

/// A universal resource type for transmitting data inside Teravolt.
#[derive(Debug, Clone)]
pub struct Resource<T>(Arc<RwLock<Box<dyn Any + Send + Sync>>>, PhantomData<T>);

impl<T> Resource<T>
where
    T: ResourceTrait + Debug,
{
    /// Get an immutable reference to the resource.
    #[tracing::instrument]
    pub async fn read(&self) -> RwLockReadGuard<'_, T> {
        RwLockReadGuard::map(self.0.read().await, |data| {
            // Theoretically, this should never panic
            data.downcast_ref::<T>().unwrap()
        })
    }

    /// Get a mutable reference to the resource.
    #[tracing::instrument]
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
    map: Arc<Mutex<HashMap<TypeId, InternalResource>>>,
}

impl Storage {
    /// Create a new Storage object.
    #[tracing::instrument]
    pub fn new() -> Self {
        Self {
            map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get a handle on a Resource within storage.
    #[tracing::instrument]
    pub fn handle<T: ResourceTrait + Debug>(&self) -> Result<Resource<T>> {
        let mut map = self.map.lock().or(error("Could not acquire lock".into()))?;

        if let Some(resource) = map.get(&TypeId::of::<T>()) {
            Ok(resource.as_typed())
        } else {
            let resource = InternalResource::new(T::default());
            map.insert(TypeId::of::<T>(), resource.clone());
            Ok(resource.as_typed())
        }
    }

    /// Clear out a resource from storage.
    #[tracing::instrument]
    pub async fn clear<T: ResourceTrait + Debug>(&self) -> Result<()> {
        let map = self.map.lock().or(error("Could not acquire lock".into()))?;
        if let Some(resource) = map.get(&TypeId::of::<T>()) {
            let data = resource.as_typed();
            *data.write().await = T::default();
        }
        Ok(())
    }
}
