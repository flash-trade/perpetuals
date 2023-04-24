// admin instructions
pub mod admin;
pub mod keeper;
pub mod user;

// public instructions

// bring everything in scope
pub use {admin::*, keeper::*, user::*};
