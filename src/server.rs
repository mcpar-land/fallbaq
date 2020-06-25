use actix_files;
use actix_web::error::ErrorNotFound;
use actix_web::{web, Error, HttpRequest, HttpResponse};
use colored::*;
use std::path::Path;
use std::path::PathBuf;

use crate::files::fetch;

#[derive(Clone)]
pub struct Server {
	pub paths: Vec<Box<Path>>,
	pub port: i32,
}

impl Server {
	pub fn new(paths: Vec<Box<Path>>, port: i32) -> Server {
		Server {
			paths: paths,
			port: port,
		}
	}
}

pub async fn index(
	data: web::Data<Server>,
	req: HttpRequest,
) -> Result<HttpResponse, Error> {
	let path: PathBuf = req.match_info().query("filename").parse::<PathBuf>()?;

	match fetch(path.as_path(), &data.paths) {
		Some((i, p)) => {
			println!(
				"{} {} {} {} {} {}",
				"✓".green(),
				path.to_str().unwrap().green(),
				"⟶ ".magenta().bold(),
				data.paths[i].to_str().unwrap().yellow(),
				"⟶ ".magenta().bold(),
				p.to_str().unwrap().blue()
			);
			let file = actix_files::NamedFile::open(p)?;
			file.into_response(&req)
		}
		None => {
			println!("{} {}", "✘ ".red().bold(), path.to_str().unwrap().red());
			Err(ErrorNotFound("Not found"))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use actix_web::test;
	use actix_web::{web, App};
	use std::error::Error;
	use std::fs;
	use std::io::Write;
	use std::path::Path;
	use tempfile;

	fn serv() -> Result<(Vec<Box<Path>>, impl FnOnce() -> ()), Box<dyn Error>> {
		let folder = tempfile::tempdir()?;

		fs::create_dir(folder.path().join("subdir_1"))?;
		fs::create_dir(folder.path().join("subdir_1/double_subdir"))?;
		fs::create_dir(folder.path().join("subdir_2"))?;
		fs::create_dir(folder.path().join("subdir_2/double_subdir"))?;

		fs::File::create(folder.path().join("subdir_1/nice.txt"))?
			.write_all("nice!".as_bytes())?;

		fs::File::create(folder.path().join("subdir_1/double_subdir/okay.txt"))?
			.write_all("okay!".as_bytes())?;

		fs::File::create(folder.path().join("subdir_2/cool"))?
			.write_all("kinda cool!".as_bytes())?;

		fs::File::create(folder.path().join("subdir_2/double_subdir/wow.txt"))?
			.write_all("wow!".as_bytes())?;

		let path1 = folder.path().join("subdir_1");
		let path2 = folder.path().join("subdir_2");
		let paths: Vec<Box<Path>> =
			vec![path1.into_boxed_path(), path2.into_boxed_path()];

		Ok((paths, || {
			folder.close().unwrap();
		}))
	}

	#[actix_rt::test]
	async fn test_req() {
		let (paths, close) = serv().unwrap();

		let tests = vec![
			("nice.txt", false, 200),
			("cool", false, 200),
			("double_subdir/okay.txt", false, 200),
			("double_subdir/wow.txt", false, 200),
			("does_not_exist.txt", true, 404),
		];

		for (path, error, code) in tests {
			let req = test::TestRequest::get()
				.param("filename", path)
				.to_http_request();

			let res = index(
				web::Data::new(Server {
					paths: paths.clone(),
					port: 1000,
				}),
				req,
			)
			.await;
			println!("{:?}", res);

			if !error {
				assert_eq!(res.unwrap().status(), code);
			} else {
				let err = res.unwrap_err();
				println!("{:?}", err);
			}
		}

		close();
	}

	#[actix_rt::test]
	async fn test_integration() {
		let (paths, close) = serv().unwrap();

		let tests: Vec<(&str, bool, u16)> = vec![
			("nice.txt", false, 200),
			("cool", false, 200),
			("double_subdir/okay.txt", false, 200),
			("double_subdir/wow.txt", false, 200),
			("double_subdir/../nice.txt", true, 404),
			("does_not_exist.txt", true, 404),
			("double_subdir", true, 404),
			("double_subdir/does_not_exist.txt", true, 404),
		];

		let mut app = test::init_service(
			App::new()
				.data(Server {
					paths: paths,
					port: 1000,
				})
				.route(r"/{filename:.*}", web::get().to(index)),
		)
		.await;

		for (path, _, code) in tests {
			let uri = format!("/{}", path);
			let req = test::TestRequest::with_uri(uri.as_str()).to_request();
			let res = test::call_service(&mut app, req).await;

			println!("{:?} should return {}", uri, code);
			match res.response().body().as_ref() {
				Some(a) => {
					println!("body: {:?}", a);
				}
				None => {
					println!("No body");
				}
			};

			assert_eq!(res.response().status(), code);
		}

		close();
	}
}
