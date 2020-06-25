mod files;
mod server;

use crate::server::{index, Server};
use actix_web::{web, App, HttpServer};
use clap::{App as ClapApp, Arg};
use colored::*;
use std::path::Path;
use std::path::PathBuf;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
	let c = ClapApp::new("fallbaq")
		.arg(Arg::with_name("paths").required(true).min_values(1))
		.arg(
			Arg::with_name("port")
				.long("port")
				.short("p")
				.default_value("8000"),
		)
		.get_matches();

	let path_strs: Vec<&str> = c.values_of("paths").unwrap().collect();

	let paths: Vec<Box<Path>> = path_strs
		.iter()
		.map(|&s| PathBuf::from(s).into_boxed_path())
		.collect();

	let server =
		Server::new(paths, c.value_of("port").unwrap().parse::<i32>().unwrap());

	let binding = format!("127.0.0.1:{}", server.port);

	println!(
		"{}",
		format!("\nRunning server on port {}\n", c.value_of("port").unwrap())
			.cyan()
	);
	HttpServer::new(move || {
		App::new()
			.data(server.clone())
			.route("/{filename:.*}", web::get().to(index))
	})
	.bind(binding)?
	.run()
	.await?;

	Ok(())
}
