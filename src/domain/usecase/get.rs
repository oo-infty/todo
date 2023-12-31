use chrono::NaiveDateTime;
use snafu::prelude::*;

use crate::domain::entity::{Priority, TagSet};
use crate::repository::item::{GetError, Pool};

pub struct Request {
    pub id: u64,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response {
    pub id: u64,
    pub summary: String,
    pub content: String,
    pub deadline: NaiveDateTime,
    pub tags: TagSet,
    pub priority: Priority,
}

#[derive(Debug, PartialEq, Eq, Snafu)]
pub enum GetItemError {
    #[snafu(display("Target isn't found"))]
    NotFound,
}

pub fn execute(pool: &dyn Pool, request: Request) -> Result<Response, GetItemError> {
    match pool.get(request.id) {
        Ok(item) => Ok(Response {
            id: item.id(),
            summary: item.summary().to_owned(),
            content: item.content().to_owned(),
            deadline: *item.deadline(),
            tags: item.tags().clone(),
            priority: item.priority().clone(),
        }),
        Err(GetError::NotFound) => Err(GetItemError::NotFound),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::domain::entity::Item;
    use crate::repository::item::MemoryPool;

    use super::*;

    #[test]
    fn it_should_return_the_corresponding_item_when_getting_item() {
        let item = Item::new_test();
        let id = item.id();

        let mut map = HashMap::new();
        let _ = map.insert(id, item.clone());
        let pool: Box<dyn Pool> = Box::new(MemoryPool::from(map));

        let request = Request { id };
        let res = execute(pool.as_ref(), request);

        let response = Response {
            id,
            summary: item.summary().to_owned(),
            content: item.content().to_owned(),
            deadline: *item.deadline(),
            tags: item.tags().clone(),
            priority: item.priority().clone(),
        };

        assert_eq!(res, Ok(response.clone()));

        // Do twice
        let request = Request { id };
        let res = execute(pool.as_ref(), request);
        assert_eq!(res, Ok(response));
    }

    #[test]
    fn it_should_return_not_found_error_when_the_target_does_not_exist() {
        let pool: Box<dyn Pool> = Box::new(MemoryPool::new());
        let request = Request { id: 0u64 };
        let res = execute(pool.as_ref(), request);
        assert_eq!(res, Err(GetItemError::NotFound));
    }
}
