#[macro_use]
extern crate rustler;
extern crate lazy_static;

mod bucket;
mod configuration;
mod sorted_set;
mod supported_term;

use configuration::Configuration;
use rustler::resource::ResourceArc;
use rustler::{Env, Error, NifResult, Term};
use sorted_set::SortedSet;
use std::sync::Mutex;
use supported_term::SupportedTerm;

mod atoms {
    atoms! {
        // Common Atoms
        ok,
        error,

        // Resource Atoms
        bad_reference,
        lock_fail,

        // Success Atoms
        added,
        duplicate,
        removed,

        // Error Atoms
        unsupported_type,
        not_found,
        index_out_of_bounds,
        max_bucket_size_exceeded,
    }
}

init! {
    "Elixir.Discord.SortedSet.NifBridge",
    [
        add,
        append_bucket,
        at,
        debug,
        empty,
        empty,
        find_index,
        new,
        remove,
        size,
        slice,
        to_list,
    ],
    load = load
}

pub struct SortedSetResource(Mutex<SortedSet>);

#[derive(Debug, PartialEq)]
pub enum AddResult {
    Added(usize),
    Duplicate(usize),
}

#[derive(Debug, PartialEq)]
pub enum RemoveResult {
    Removed(usize),
    NotFound,
}

#[derive(Debug, PartialEq)]
pub enum FindResult {
    Found {
        bucket_idx: usize,
        inner_idx: usize,
        idx: usize,
    },
    NotFound,
}

#[derive(Debug, PartialEq)]
pub enum AppendBucketResult {
    Ok,
    MaxBucketSizeExceeded,
}

fn load(env: Env, _info: Term) -> bool {
    resource!(SortedSetResource, env);
    true
}

#[rustler::nif]
fn empty(
    initial_item_capacity: usize,
    max_bucket_size: usize,
) -> NifResult<ResourceArc<SortedSetResource>> {
    let initial_set_capacity: usize = (initial_item_capacity / max_bucket_size) + 1;

    let configuration = Configuration {
        max_bucket_size,
        initial_set_capacity,
    };

    let resource = ResourceArc::new(SortedSetResource(Mutex::new(SortedSet::empty(
        configuration,
    ))));

    Ok(resource)
}

#[rustler::nif]
fn new(
    initial_item_capacity: usize,
    max_bucket_size: usize,
) -> NifResult<ResourceArc<SortedSetResource>> {
    let initial_set_capacity: usize = (initial_item_capacity / max_bucket_size) + 1;

    let configuration = Configuration {
        max_bucket_size,
        initial_set_capacity,
    };

    let resource = ResourceArc::new(SortedSetResource(Mutex::new(SortedSet::new(configuration))));

    Ok(resource)
}

#[rustler::nif]
fn append_bucket(
    resource: ResourceArc<SortedSetResource>,
    items: Vec<SupportedTerm>,
) -> NifResult<()> {
    let mut set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    match set.append_bucket(items) {
        AppendBucketResult::Ok => Ok(()),
        AppendBucketResult::MaxBucketSizeExceeded => {
            Err(Error::Term(Box::new(atoms::max_bucket_size_exceeded())))
        }
    }
}

#[rustler::nif]
fn add(resource: ResourceArc<SortedSetResource>, item: SupportedTerm) -> NifResult<usize> {
    let mut set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    match set.add(item) {
        AddResult::Added(idx) | AddResult::Duplicate(idx) => Ok(idx),
    }
}

#[rustler::nif]
fn remove(resource: ResourceArc<SortedSetResource>, item: SupportedTerm) -> NifResult<usize> {
    let mut set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    match set.remove(&item) {
        RemoveResult::Removed(idx) => Ok(idx),
        RemoveResult::NotFound => Err(Error::Term(Box::new(atoms::not_found()))),
    }
}

#[rustler::nif]
fn size(resource: ResourceArc<SortedSetResource>) -> NifResult<usize> {
    let set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    Ok(set.size())
}

#[rustler::nif]
fn to_list(resource: ResourceArc<SortedSetResource>) -> NifResult<Vec<SupportedTerm>> {
    let set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    Ok(set.to_vec())
}

#[rustler::nif]
fn at(resource: ResourceArc<SortedSetResource>, index: usize) -> NifResult<SupportedTerm> {
    let set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    match set.at(index) {
        None => return Err(Error::Term(Box::new(atoms::index_out_of_bounds()))),
        // TODO
        Some(value) => Ok(value.clone()),
    }
}

#[rustler::nif]
fn slice(resource: ResourceArc<SortedSetResource>, start: usize, amount: usize) -> NifResult<Vec<SupportedTerm>> {
    let set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    Ok(set.slice(start, amount))
}

#[rustler::nif]
fn find_index<'a>(resource: ResourceArc<SortedSetResource>, item: SupportedTerm) -> NifResult<usize> {
    let set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    match set.find_index(&item) {
        FindResult::Found {
            bucket_idx: _,
            inner_idx: _,
            idx,
        } => Ok(idx),
        FindResult::NotFound => Err(Error::Term(Box::new(atoms::not_found()))),
    }
}

#[rustler::nif]
fn debug(resource: ResourceArc<SortedSetResource>) -> NifResult<String> {
    let set = match resource.0.try_lock() {
        Err(_) => return Err(Error::Term(Box::new(atoms::lock_fail()))),
        Ok(guard) => guard,
    };

    Ok(set.debug())
}
