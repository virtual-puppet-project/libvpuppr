use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    path::Path,
};

use chrono::{Datelike, Timelike};
use gluesql::prelude::*;
use godot::{
    engine::{global::Error as GodotError, ProjectSettings},
    prelude::*,
};
use log::{debug, error};

use crate::model::dao::ToVariantDao;

pub const DB_PATH: &str = "user://db";

const INIT_SQL: &str = include_str!("../resources/sql/init.sql");

#[derive(Debug)]
pub enum Error {
    ExecutionError {
        command: String,
        error: gluesql::prelude::Error,
    },
    TooManyStatements(usize),
    SelectFailure,
    InsertFailure,
    UpdateFailure,
    DeleteFailure,
    CreateTableFailure,
    DropTableFailure,
    AlterTableFailure,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExecutionError { command, error } => {
                write!(f, "Failed to execute: {command}\nOriginal error: {error}")
            }
            Self::TooManyStatements(v) => write!(f, "Found {v} statements, declining to execute"),
            Self::SelectFailure => write!(f, "Select failure"),
            Self::InsertFailure => write!(f, "Insert failure"),
            Self::UpdateFailure => write!(f, "Update failure"),
            Self::DeleteFailure => write!(f, "Delete failure"),
            Self::CreateTableFailure => write!(f, "Create table failure"),
            Self::DropTableFailure => write!(f, "Drop table failure"),
            Self::AlterTableFailure => write!(f, "Alter table failure"),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

#[derive(GodotClass)]
pub struct Database {
    db: Glue<SledStorage>,
}

impl Deref for Database {
    type Target = Glue<SledStorage>;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}

impl DerefMut for Database {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.db
    }
}

#[godot_api]
impl RefCountedVirtual for Database {
    // NOTE calling `new` is not allowed
    #[allow(unreachable_code)]
    fn init(_base: godot::obj::Base<Self::Base>) -> Self {
        panic!("Use create instead of new for database safety");

        let storage = SledStorage::new(
            ProjectSettings::singleton()
                .globalize_path(DB_PATH.to_string().into())
                .to_string()
                .as_str(),
        )
        .unwrap();

        Self {
            db: Glue::new(storage),
        }
    }
}

#[godot_api]
impl Database {
    #[func]
    fn create() -> Option<Gd<Database>> {
        debug!("Create database");

        let db_path = ProjectSettings::singleton()
            .globalize_path(DB_PATH.to_string().into())
            .to_string();
        let should_init = if Path::new(&db_path).exists() {
            false
        } else {
            true
        };

        let storage = match SledStorage::new(&db_path) {
            Ok(v) => v,
            Err(e) => {
                error!("{e}");
                return None;
            }
        };

        let mut glue = Glue::new(storage);

        if should_init {
            debug!("Initializing database");
            if let Err(e) = glue.execute(INIT_SQL) {
                error!("Unable to initialize database: {e}\nDatabase is likely in a zombie state");
            }
        }

        Some(Gd::new(Self { db: glue }))
    }

    /// Execute a sql command, discard the results, and return a success code.
    #[func(rename = run)]
    fn run_bound(&mut self, command: GodotString) -> GodotError {
        debug!("Running sql command: {command}");

        match self.run(command.to_string()) {
            Ok(_) => GodotError::OK,
            Err(e) => {
                error!("{e}");
                GodotError::ERR_INVALID_PARAMETER
            }
        }
    }

    /// Run a select query.
    #[func(rename = select)]
    pub fn select_bound(&mut self, command: GodotString) -> Array<Array<Variant>> {
        debug!("Selecting sql: {command}");

        if let Ok(v) = self.select(command.to_string()) {
            return Array::from_iter(
                v.iter()
                    .map(|v| Array::from_iter(v.iter().map(Value::to_variant))),
            );
        }

        Array::new()
    }

    /// Run an insert statement.
    #[func(rename = insert)]
    fn insert_bound(&mut self, command: GodotString) -> GodotError {
        debug!("Inserting sql: {command}");

        if let Err(e) = self.insert(command.to_string()) {
            error!("{e}");
            return GodotError::ERR_DATABASE_CANT_WRITE;
        }

        GodotError::OK
    }

    /// Run an update statement.
    #[func(rename = update)]
    fn update_bound(&mut self, command: GodotString) -> GodotError {
        debug!("Updating sql: {command}");

        if let Err(e) = self.update(command.to_string()) {
            error!("{e}");
            return GodotError::ERR_DATABASE_CANT_WRITE;
        }

        GodotError::OK
    }

    /// Run a delete statement.
    #[func(rename = delete)]
    fn delete_bound(&mut self, command: GodotString) -> GodotError {
        debug!("Deleting sql: {command}");

        if let Err(e) = self.delete(command.to_string()) {
            error!("{e}");
            return GodotError::ERR_DATABASE_CANT_WRITE;
        }

        GodotError::OK
    }

    /// Run a create table statement.
    #[func(rename = create_table)]
    fn create_table_bound(&mut self, command: GodotString) -> GodotError {
        debug!("Creating table sql: {command}");

        if let Err(e) = self.create_table(command.to_string()) {
            error!("{e}");
            return GodotError::ERR_DATABASE_CANT_WRITE;
        }

        GodotError::OK
    }

    /// Run a drop table statement.
    #[func(rename = drop_table)]
    fn drop_table_bound(&mut self, command: GodotString) -> GodotError {
        debug!("Dropping table sql: {command}");

        if let Err(e) = self.drop_table(command.to_string()) {
            error!("{e}");
            return GodotError::ERR_DATABASE_CANT_WRITE;
        }

        GodotError::OK
    }

    /// Run an alter table statement.
    #[func(rename = alter_table)]
    fn alter_table_bound(&mut self, command: GodotString) -> GodotError {
        debug!("Altering table sql: {command}");

        if let Err(e) = self.alter_table(command.to_string()) {
            error!("{e}");
            return GodotError::ERR_DATABASE_CANT_WRITE;
        }

        GodotError::OK
    }
}

impl Database {
    /// Execute a sql command and return the raw results.
    pub fn run(&mut self, command: impl AsRef<str>) -> Result<Vec<Payload>> {
        let command = command.as_ref();
        self.execute(command).map_err(|error| {
            error!("Unable to execute:\n{}", command);
            Error::ExecutionError {
                command: command.to_string(),
                error,
            }
        })
    }

    /// Run a select query. The results will be assumed to be from a select statement.
    pub fn select(&mut self, command: impl AsRef<str>) -> Result<Vec<Vec<Value>>> {
        let mut payloads = match self.run(command.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        if payloads.len() > 1 {
            error!("Found too many statements, unable to select");
            return Err(Error::TooManyStatements(payloads.len()));
        }

        if let Some(payload) = payloads.pop() {
            let Payload::Select { rows, .. } = payload else {
                error!("Unhandled payload data: {payload:?}");
                return Err(Error::SelectFailure);
            };

            return Ok(rows);
        }

        Ok(vec![])
    }

    /// Run an insert statement. The results will be assumed to be from an insert statement.
    pub fn insert(&mut self, command: impl AsRef<str>) -> Result<()> {
        let payloads = match self.run(command.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        if payloads.len() < 1 {
            error!("No payloads returned, insertion probably failed");
            return Err(Error::InsertFailure);
        }

        Ok(())
    }

    /// Run an update statement. The results will be assumed to be from an update statement.
    pub fn update(&mut self, command: impl AsRef<str>) -> Result<()> {
        let payloads = match self.run(command.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        if payloads.len() < 1 {
            error!("No payloads returned, update probably failed");
            return Err(Error::UpdateFailure);
        }

        Ok(())
    }

    /// Run a delete statement. The results will be assumed to be from a delete statement.
    pub fn delete(&mut self, command: impl AsRef<str>) -> Result<()> {
        let payloads = match self.run(command.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        if payloads.len() < 1 {
            error!("No payloads returned, delete probably failed");
            return Err(Error::DeleteFailure);
        }

        Ok(())
    }

    /// Run a create table statement. The results will be assumed to be from a create table statement.
    pub fn create_table(&mut self, command: impl AsRef<str>) -> Result<()> {
        let payloads = match self.run(command.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        if payloads.len() < 1 {
            error!("No payloads returned, create table probably failed");
            return Err(Error::CreateTableFailure);
        }

        Ok(())
    }

    /// Run a drop table statement. The results will be assumed to be from a drop table statement.
    pub fn drop_table(&mut self, command: impl AsRef<str>) -> Result<()> {
        let payloads = match self.run(command.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        if payloads.len() < 1 {
            error!("No payloads returned, drop table probably failed");
            return Err(Error::DropTableFailure);
        }

        Ok(())
    }

    /// Run an alter table statement. The results will be assumed to be from an alter table statement.
    pub fn alter_table(&mut self, command: impl AsRef<str>) -> Result<()> {
        let payloads = match self.run(command.as_ref()) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        if payloads.len() < 1 {
            error!("No payloads returned, alter table probably failed");
            return Err(Error::AlterTableFailure);
        }

        Ok(())
    }
}
