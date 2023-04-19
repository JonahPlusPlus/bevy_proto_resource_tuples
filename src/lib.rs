//! A prototype for working with [`Resource`]s in groups.
//!
//! [Initial PR](https://github.com/bevyengine/bevy/pull/8126)
//!
//! ## Usage
//! ```no_run
//! use bevy::prelude::*;
//! use bevy_proto_resource_tuples::*;
//!
//! #[derive(Resource)]
//! struct ResourceA;
//!
//! #[derive(Resource)]
//! struct ResourceB;
//!
//! #[derive(Resource, Default)]
//! struct ResourceC;
//!
//! #[derive(Resource, Default)]
//! struct ResourceD;
//!
//! fn main() {
//!     App::new()
//!         .insert_resources((ResourceA, ResourceB))
//!         .init_resources::<(ResourceC, ResourceD)>()
//!         .run();
//! }
//! ```
//!
//! ## Motivation
//!
//! With Bevy 0.10 and the upcoming 0.11 releases, it is possible to use `add_systems` to add multiple systems at once, instead of making multiple calls to `add_system`.
//! Likewise, there are changes underway to add multiple plugins at once.
//!
//! Just as those interfaces were changed to minimize calls, `init_resource` and `insert_resource` calls can also be minimized, in order for the interface to be homogenous.
//!
//! This is a bit controversial; the primary concern is that by grouping resources, it in a way implies that resources form "groups", rather than being separate objects.
//! This differs from groups of systems, where there are operations over groups of systems, while there aren't any operations to be done over groups of resources, since they are each unique pieces of data.
//!
//! It appears that some view `add_systems` as a tool only for working with groups of systems that have common attributes (and then making separate calls for unique systems), while others see it as a tool for adding systems in batch.
//!
//! In reality, there is nothing stopping anyone from using `add_systems` either way. But from the perspective of those who use it simply to batch calls and lower LOC, the lack of `insert_resources` and `init_resources` feels rather inconsistent.
//! Also, if these methods were added, there's nothing stopping users from making separate calls like before.
//!
//! tl;dr: The benefit of these methods is that they bring consistency and usability without preventing users from doing stuff in the old style or mixing styles to what feels appropriate.
//!
//! tl;dr for the tl;dr: More user freedom.
//!
//! ## Limitations
//!
//! Because this prototype is a separate crate, I can't implement the traits for the base case (`P0`) due to orphan rules.
//! So when working with a single resource, it's necessary to call `init_resource`/`insert_resource`.
//!
//! ## Patterns
//!
//! The following are some patterns enabled by these changes. Whether or not they are useful is up to users to discover in practice.
//! Either way, these are very niche, but perhaps they have some use cases.
//!
//! ```ignore
//! // It's now possible to init many resources and get their ids all at once.
//! let [a, b, c] = world.init_resources::<(A, B, C)>();
//! ```
//!
//! ```ignore
//! // It's possible to create type aliases for multiple resources.
//! type MyResources<T> = (Foo<T>, Bar<T>);
//!
//! app.init_resources(MyResources<i32>);
//! ```

use std::marker::PhantomData;

use bevy_app::App;
use bevy_ecs::{
    component::ComponentId,
    system::{Command, Commands, Resource},
    world::{FromWorld, World},
};

/// Resources that can be initialized in the [`World`] together.
pub trait InitResources: Send + Sync + 'static {
    type IDS;

    fn init_resources(world: &mut World) -> Self::IDS;
}

/// Resources that can be inserted into the [`World`] together.
pub trait InsertResources: Send + Sync + 'static {
    fn insert_resources(self, world: &mut World);
}

/// Extends [`World`] with `init_resources`.
pub trait WorldInitResources {
    /// Initializes new resources and returns a vector of the [`ComponentId`]s created for them.
    ///
    /// If a resource already exists, nothing happens.
    ///
    /// The value given by the [`FromWorld::from_world`] method will be used.
    /// Note that any resource with the [`Default`] trait automatically implements [`FromWorld`],
    /// and those default values will be here instead.
    fn init_resources<R: InitResources>(&mut self) -> R::IDS;
}

impl WorldInitResources for World {
    fn init_resources<R: InitResources>(&mut self) -> R::IDS {
        R::init_resources(self)
    }
}

/// Extends [`App`] with `init_resources`.
pub trait AppInitResources {
    /// Initialize a [`Resource`] with standard starting values by adding it to the [`World`].
    ///
    /// If the [`Resource`] already exists, nothing happens.
    ///
    /// The [`Resource`] must implement the [`FromWorld`] trait.
    /// If the [`Default`] trait is implemented, the [`FromWorld`] trait will use
    /// the [`Default::default`] method to initialize the [`Resource`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_app::prelude::*;
    /// # use bevy_ecs::prelude::*;
    /// #
    /// #[derive(Resource)]
    /// struct MyCounter {
    ///     counter: usize,
    /// }
    ///
    /// impl Default for MyCounter {
    ///     fn default() -> MyCounter {
    ///         MyCounter {
    ///             counter: 100
    ///         }
    ///     }
    /// }
    ///
    /// #[derive(Resource)]
    /// struct MyValue {
    ///     value: f32,
    /// }
    ///
    /// impl Default for MyValue {
    ///     fn default() -> MyValue {
    ///         MyValue {
    ///             value: 20.0
    ///         }
    ///     }
    /// }
    ///
    /// App::new()
    ///     .init_resources::<(MyCounter, MyValue)>();
    /// ```
    fn init_resources<R: InitResources>(&mut self) -> &mut Self;
}

impl AppInitResources for App {
    fn init_resources<R: InitResources>(&mut self) -> &mut Self {
        self.world.init_resources::<R>();
        self
    }
}

/// Extends [`Commands`] with `init_resources`.
pub trait CommandsInitResources {
    /// Pushes a [`Command`] to the queue for inserting a [`Resource`] in the [`World`] with an inferred value.
    ///
    /// The inferred value is determined by the [`FromWorld`] trait of the resource.
    /// When the command is applied,
    /// if the resource already exists, nothing happens.
    ///
    /// See [`World::init_resource`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_ecs::prelude::*;
    /// #
    /// # #[derive(Resource, Default)]
    /// # struct PlayerScoreboard {
    /// #     current_score: u32,
    /// #     high_score: u32,
    /// # }
    /// #
    /// # #[derive(Resource, Default)]
    /// # struct EnemyScoreboard {
    /// #     current_score: u32,
    /// #     high_score: u32,
    /// # }
    /// #
    /// # fn initialise_scoreboards(mut commands: Commands) {
    /// commands.init_resources::<(PlayerScoreboard, EnemyScoreboard)>();
    /// # }
    /// # bevy_ecs::system::assert_is_system(initialise_scoreboards);
    /// ```
    fn init_resources<R: InitResources>(&mut self);
}

impl CommandsInitResources for Commands<'_, '_> {
    fn init_resources<R: InitResources>(&mut self) {
        self.add(InitResourcesCommand::<R>::new())
    }
}

/// [`Command`] for `init_resources`.
pub struct InitResourcesCommand<R: InitResources> {
    _phantom: PhantomData<R>,
}

impl<R: InitResources> Command for InitResourcesCommand<R> {
    fn write(self, world: &mut World) {
        world.init_resources::<R>();
    }
}

impl<R: InitResources> InitResourcesCommand<R> {
    /// Creates a [`Command`] which will insert a default created [`Resource`] into the [`World`]
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData::<R>,
        }
    }
}

/// Extends [`World`] with `insert_resources`.
pub trait WorldInsertResources {
    fn insert_resources<R: InsertResources>(&mut self, resources: R);
}

impl WorldInsertResources for World {
    /// Inserts a new resource with the given `value`.
    ///
    /// Resources are "unique" data of a given type.
    /// If you insert a resource of a type that already exists,
    /// you will overwrite any existing data.
    fn insert_resources<R: InsertResources>(&mut self, resources: R) {
        resources.insert_resources(self);
    }
}

/// Extends [`App`] with `insert_resources`.
pub trait AppInsertResources {
    /// Inserts a [`Resource`] to the current [`App`] and overwrites any [`Resource`] previously added of the same type.
    ///
    /// A [`Resource`] in Bevy represents globally unique data. [`Resource`]s must be added to Bevy apps
    /// before using them. This happens with [`insert_resources`](Self::insert_resources).
    ///
    /// See `init_resources` for [`Resource`]s that implement [`Default`] or [`FromWorld`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use bevy_app::prelude::*;
    /// # use bevy_ecs::prelude::*;
    /// #
    /// #[derive(Resource)]
    /// struct MyCounter {
    ///     counter: usize,
    /// }
    ///
    /// #[derive(Resource)]
    /// struct MyValue {
    ///     value: f32,
    /// }
    ///
    /// App::new()
    ///    .insert_resources((MyCounter { counter: 0 }, MyValue { value: 1.0 }));
    /// ```
    fn insert_resources<R: InsertResources>(&mut self, resources: R) -> &mut Self;
}

impl AppInsertResources for App {
    fn insert_resources<R: InsertResources>(&mut self, resources: R) -> &mut Self {
        self.world.insert_resources(resources);
        self
    }
}

/// Extends [`Commands`] with `insert_resources`.
pub trait CommandsInsertResources {
    /// Pushes a [`Command`] to the queue for inserting a [`Resource`] in the [`World`] with a specific value.
    ///
    /// This will overwrite any previous value of the same resource type.
    ///
    /// See [`World::insert_resources`] for more details.
    ///
    /// # Example
    ///
    /// ```
    /// # use bevy_ecs::prelude::*;
    /// #
    /// # #[derive(Resource)]
    /// # struct PlayerScoreboard(u32);
    /// #
    /// # #[derive(Resource)]
    /// # struct EnemyScoreboard(u32);
    /// #
    /// # fn system(mut commands: Commands) {
    /// commands.insert_resources((
    ///     PlayerScoreboard(0),
    ///     EnemyScoreboard(0),
    /// ));
    /// # }
    /// # bevy_ecs::system::assert_is_system(system);
    /// ```
    fn insert_resources<R: InsertResources>(&mut self, resources: R);
}

impl CommandsInsertResources for Commands<'_, '_> {
    fn insert_resources<R: InsertResources>(&mut self, resources: R) {
        self.add(InsertResourcesCommand { resources });
    }
}

/// [`Command`] for `insert_resources`.
pub struct InsertResourcesCommand<R: InsertResources> {
    pub resources: R,
}

impl<R: InsertResources> Command for InsertResourcesCommand<R> {
    fn write(self, world: &mut World) {
        world.insert_resources(self.resources);
    }
}

bevy_proto_resource_tuples_macros::impl_resource_apis!();
