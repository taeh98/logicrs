#![feature(const_fn_floating_point_arithmetic)]
#![feature(let_chains)]
#![feature(result_flattening)]
#![feature(const_trait_impl)]
#![feature(if_let_guard)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

use adw::prelude::ApplicationExtManual;

use application::Application;

mod application;
mod config;
mod export;
mod fatal;
mod id;
mod project;
mod renderer;
mod simulator;
mod ui;

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
