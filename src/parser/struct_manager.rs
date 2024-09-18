use std::{borrow::Cow, collections::HashMap};

use crate::{types::{HasKey, Struct, Type}, IdentifierType, NoHashMap, ParserError};
use super::IdMapper;

#[derive(Debug, Clone)]
pub struct StructBuilder<'a> {
    pub fields: Vec<Type>,
    pub mapper: IdMapper<'a>
}

pub struct StructManager<'a> {
    parent: Option<&'a StructManager<'a>>,
    // All structs registered in the manager
    structures: NoHashMap<StructBuilder<'a>>,
    // mapper to map each string name into a unique identifier
    mapper: IdMapper<'a>
}

impl<'a> StructManager<'a> {
    // Create a new struct manager
    pub fn new() -> Self {
        StructManager {
            parent: None,
            structures: HashMap::default(),
            mapper: IdMapper::new()
        }
    }

    pub fn with_parent(parent: &'a StructManager<'a>) -> Self {
        StructManager {
            parent: Some(parent),
            structures: HashMap::default(),
            mapper: IdMapper::with_parent(&parent.mapper)
        }
    }

    // register a new struct in the manager
    pub fn add(&mut self, name: Cow<'a, str>, builder: StructBuilder<'a>) -> Result<(), ParserError<'a>> {
        if self.mapper.has_variable(&name) {
            return Err(ParserError::StructNameAlreadyUsed(name.into_owned()));
        }

        let id = self.mapper.register(name)?;
        self.structures.insert(id, builder);

        Ok(())
    }

    // Same as `add` but returns its identifier and the final struct
    pub fn build_struct(&mut self, name: Cow<'a, str>, builder: StructBuilder<'a>) -> Result<(IdentifierType, Struct), ParserError<'a>> {
        if self.mapper.has_variable(&name) {
            return Err(ParserError::StructNameAlreadyUsed(name.into_owned()));
        }

        let id = self.mapper.register(name)?;
        let s = Struct { fields: builder.fields.clone() };
        self.structures.insert(id, builder);

        Ok((id, s))
    }

    // Get the mapping of a struct by name
    pub fn get_mapping(&self, name: &str) -> Result<IdentifierType, ParserError<'a>> {
        if let Some(parent) = self.parent {
            if let Ok(id) = parent.get_mapping(name) {
                return Ok(id);
            }
        }

        self.mapper.get(name)
    }

    // Get a struct by name
    pub fn get(&self, name: &IdentifierType) -> Result<&StructBuilder<'a>, ParserError<'a>> {
        self.structures.get(name).ok_or_else(|| ParserError::StructNotFound(name.clone()))
    }

    // Convert the struct manager into a hashmap of structs
    pub fn finalize(self) -> NoHashMap<Struct> {
        self.structures.into_iter().map(|(k, v)| (k, Struct { fields: v.fields })).collect()
    }
}

impl HasKey<IdentifierType> for StructManager<'_> {
    fn has(&self, key: &IdentifierType) -> bool {
        if let Some(parent) = self.parent {
            if parent.has(key) {
                return true;
            }
        }

        self.structures.contains_key(key)
    }
}