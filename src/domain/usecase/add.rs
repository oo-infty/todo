use chrono::prelude::*;
use snafu::prelude::*;

use crate::domain::entity::{Item, Priority, TagSet};
use crate::repository::item::{AddError, Pool};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub summary: String,
    pub content: String,
    pub deadline: NaiveDateTime,
    pub tags: TagSet,
    pub priority: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    pub id: u64,
}

#[derive(Debug, PartialEq, Eq, Snafu)]
pub enum AddItemError {
    #[snafu(display("`summary` may not be empty and `priority` should be in [-3, 3]"))]
    Invalid,
    #[snafu(display("Two same items may not exist"))]
    Conflict,
}

pub fn execute(pool: &mut dyn Pool, request: Request) -> Result<Response, AddItemError> {
    let Request {
        summary,
        content,
        deadline,
        tags,
        priority,
    } = request;
    ensure!(!summary.is_empty(), InvalidSnafu);

    let priority = match Priority::try_from(priority) {
        Ok(v) => v,
        Err(()) => return Err(AddItemError::Invalid),
    };

    let res = pool.add(Item::new(
        summary.as_str(),
        content.as_str(),
        deadline,
        tags,
        priority,
    ));

    match res {
        Ok(id) => Ok(Response { id }),
        Err(AddError::Conflict) => Err(AddItemError::Conflict),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::repository::item::MemoryPool;

    use super::*;

    #[test]
    fn it_should_return_an_id_when_creating_item_succeeded() {
        let item = Item::new_test();
        let id = item.id();

        let request = Request {
            summary: item.summary().to_owned(),
            content: item.content().to_owned(),
            deadline: *item.deadline(),
            tags: item.tags().clone(),
            priority: item.priority().value(),
        };

        let mut pool: Box<dyn Pool> = Box::new(MemoryPool::new());
        let res = execute(pool.as_mut(), request);
        assert_eq!(res, Ok(Response { id }));
    }

    #[test]
    fn it_should_return_invalid_error_when_summary_is_empty() {
        let request = Request {
            summary: String::new(),
            content: String::from("This is content."),
            deadline: get_deadline(),
            tags: HashSet::new(),
            priority: 0i32,
        };

        let mut pool: Box<dyn Pool> = Box::new(MemoryPool::new());
        let res = execute(pool.as_mut(), request);
        assert_eq!(res, Err(AddItemError::Invalid));
    }

    #[test]
    fn it_should_return_invalid_error_when_priority_is_out_of_bound() {
        let request = Request {
            summary: String::from("Test"),
            content: String::from("This is content."),
            deadline: get_deadline(),
            tags: HashSet::new(),
            priority: 10i32,
        };

        let mut pool: Box<dyn Pool> = Box::new(MemoryPool::new());
        let res = execute(pool.as_mut(), request);
        assert_eq!(res, Err(AddItemError::Invalid));
    }

    #[test]
    fn it_should_return_conflict_error_when_adding_two_same_items() {
        let request = Request {
            summary: String::from("Test"),
            content: String::from("This is content."),
            deadline: get_deadline(),
            tags: HashSet::new(),
            priority: 0i32,
        };

        let mut pool: Box<dyn Pool> = Box::new(MemoryPool::new());
        let _ = execute(pool.as_mut(), request.clone());
        let res = execute(pool.as_mut(), request);
        assert_eq!(res, Err(AddItemError::Conflict));
    }

    #[inline]
    fn get_deadline() -> NaiveDateTime {
        NaiveDateTime::parse_from_str("2023-06-17 23:20:00", "%Y-%m-%d %H:%M:%S").unwrap()
    }
}
