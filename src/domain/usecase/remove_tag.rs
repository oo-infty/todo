use snafu::prelude::*;

use crate::domain::entity::TagSet;
use crate::repository::item::{Pool, RemoveTagError as RepositoryError};

pub struct Request {
    pub id: u64,
    pub tags: TagSet,
}

#[derive(Debug, PartialEq, Eq, Snafu)]
pub enum RemoveTagError {
    #[snafu(display("No item with given id is found"))]
    ItemNotFound,
    #[snafu(display("Some tags are not found"))]
    TagNotFound,
    #[snafu(display("Some tags have already existed"))]
    Conflict,
}

pub fn execute(pool: &mut dyn Pool, request: Request) -> Result<(), RemoveTagError> {
    match pool.remove_tag(request.id, request.tags) {
        Ok(()) => Ok(()),
        Err(RepositoryError::Conflict) => Err(RemoveTagError::Conflict),
        Err(RepositoryError::ItemNotFound) => Err(RemoveTagError::ItemNotFound),
        Err(RepositoryError::TagNotFound) => Err(RemoveTagError::TagNotFound),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::domain::entity::Item;
    use crate::repository::item::MemoryPool;

    use super::*;

    #[test]
    fn it_should_return_ok_when_succeeding() {
        let mut item = Item::new_test();
        let id = item.id();
        item.add_tag("a".to_owned());
        item.add_tag("b".to_owned());

        let mut map = HashMap::new();
        let _ = map.insert(id, item);
        let mut pool: Box<dyn Pool> = Box::new(MemoryPool::from(map));

        let tags = ["a", "b"].iter().map(|&s| s.to_owned()).collect();
        let request = Request { id, tags };
        let res = execute(pool.as_mut(), request);
        assert_eq!(res, Ok(()));

        if let Ok(item) = pool.get(id) {
            assert!(!item.find_tag(&"a".to_owned()));
            assert!(!item.find_tag(&"b".to_owned()));
        } else {
            unreachable!()
        }
    }

    #[test]
    fn it_should_return_tag_not_found_but_remove_the_rest_when_tags_do_not_exist() {
        let mut item = Item::new_test();
        let id = item.id();
        item.add_tag("a".to_owned());
        item.add_tag("b".to_owned());

        let mut map = HashMap::new();
        let _ = map.insert(id, item);
        let mut pool: Box<dyn Pool> = Box::new(MemoryPool::from(map));

        let tags = ["c", "d"].iter().map(|&s| s.to_owned()).collect();
        let request = Request { id, tags };
        let res = execute(pool.as_mut(), request);
        assert_eq!(res, Err(RemoveTagError::TagNotFound));
    }

    #[test]
    fn it_should_return_item_not_found_error_when_the_target_does_not_exist() {
        let mut pool: Box<dyn Pool> = Box::new(MemoryPool::new());

        let request = Request {
            id: 0,
            tags: TagSet::new(),
        };

        let res = execute(pool.as_mut(), request);
        assert_eq!(res, Err(RemoveTagError::ItemNotFound));
    }
}
