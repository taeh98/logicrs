#![feature(const_fn_floating_point_arithmetic)]
#![feature(let_chains)]
#![feature(result_flattening)]
#![feature(const_trait_impl)]
#![feature(if_let_guard)]

mod application;
mod ui;
mod renderer;
mod simulator;
mod config;
mod fatal;
mod project;
mod export;
mod id;

#[macro_use]
extern crate log;

#[macro_use]
extern crate lazy_static;

use adw::prelude::ApplicationExtManual;
use application::Application;

trait FileExtension {
    const FILE_EXTENSION: &'static str;
    const FILE_PATTERN: &'static str;

    fn file_filter() -> gtk::FileFilter;
}

fn main() {
    env_logger::init();
    info!("Starting up LogicRs...");    
    
    let application = Application::new();
    std::process::exit(application.run());
}
