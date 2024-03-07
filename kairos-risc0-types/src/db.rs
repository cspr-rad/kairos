#[macro_use]
extern crate diesel;

use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};

use tokio_diesel::*;

