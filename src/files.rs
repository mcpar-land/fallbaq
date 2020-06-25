use std::fs::metadata;
use std::path::Path;

fn fetch_one(req: &Path, path: &Path) -> Option<Box<Path>> {
	if req.to_str().unwrap().contains("../") {
		return None;
	}

	let full_path = path.join(req);

	let is_file: bool = match metadata(full_path.as_path()) {
		Ok(r) => r.is_file(),
		Err(_) => false,
	};

	if !full_path.exists() || !is_file {
		None
	} else {
		Some(full_path.into_boxed_path())
	}
}

pub fn fetch(req: &Path, paths: &Vec<Box<Path>>) -> Option<(usize, Box<Path>)> {
	for (i, path) in paths.iter().enumerate() {
		let res = fetch_one(req, path);
		if res != None {
			return Some((i, res.unwrap()));
		}
	}
	None
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::error::Error;
	use std::fs;
	use std::path::Path;
	use tempfile;

	fn bp(f: &tempfile::TempDir, p: &str) -> Option<Box<Path>> {
		Some(f.path().join(p).into_boxed_path())
	}

	fn bpt(
		f: &tempfile::TempDir,
		p: &str,
		i: usize,
	) -> Option<(usize, Box<Path>)> {
		Some((i, bp(f, p).unwrap()))
	}

	#[test]
	fn test_fetch_one() -> Result<(), Box<dyn Error>> {
		let folder = tempfile::tempdir()?;

		fs::File::create(folder.path().join("nice.txt"))?;

		assert_eq!(
			fetch_one(Path::new("nice.txt"), folder.path()),
			bp(&folder, "nice.txt")
		);

		folder.close()?;
		Ok(())
	}
	#[test]
	fn test_fetch_one_directories() -> Result<(), Box<dyn Error>> {
		let folder = tempfile::tempdir()?;

		fs::create_dir(folder.path().join("subdir"))?;

		fs::File::create(folder.path().join("subdir/nice.txt"))?;

		assert_eq!(
			fetch_one(Path::new("subdir/nice.txt"), folder.path()),
			bp(&folder, "subdir/nice.txt")
		);

		folder.close()?;
		Ok(())
	}
	#[test]
	fn test_fetch_one_none() -> Result<(), Box<dyn Error>> {
		let folder = tempfile::tempdir()?;

		fs::create_dir(folder.path().join("subdir"))?;

		fs::File::create(folder.path().join("nice.txt"))?;

		let boxed_path = folder.path();

		assert_eq!(fetch_one(Path::new("nothing.txt"), boxed_path), None);
		assert_eq!(fetch_one(Path::new("doom/nothing.txt"), boxed_path), None);
		assert_eq!(fetch_one(Path::new("subdir"), boxed_path), None);
		assert_eq!(
			fetch_one(Path::new("nice.txt"), boxed_path),
			bp(&folder, "nice.txt")
		);

		folder.close()?;
		Ok(())
	}
	#[test]
	fn test_fetch() -> Result<(), Box<dyn Error>> {
		let folder = tempfile::tempdir()?;

		fs::create_dir(folder.path().join("subdir_1"))?;
		fs::create_dir(folder.path().join("subdir_1/double_subdir"))?;
		fs::create_dir(folder.path().join("subdir_2"))?;
		fs::create_dir(folder.path().join("subdir_2/double_subdir"))?;

		fs::File::create(folder.path().join("subdir_1/nice.txt"))?;

		fs::File::create(folder.path().join("subdir_1/double_subdir/okay.txt"))?;

		fs::File::create(folder.path().join("subdir_2/cool"))?;

		fs::File::create(folder.path().join("subdir_2/double_subdir/wow.txt"))?;

		let path1 = folder.path().join("subdir_1");
		let path2 = folder.path().join("subdir_2");
		let paths: Vec<Box<Path>> =
			vec![path1.into_boxed_path(), path2.into_boxed_path()];

		assert_eq!(
			fetch(Path::new("nice.txt"), &paths),
			bpt(&folder, "subdir_1/nice.txt", 0)
		);
		assert_eq!(
			fetch(Path::new("double_subdir/okay.txt"), &paths),
			bpt(&folder, "subdir_1/double_subdir/okay.txt", 0)
		);
		assert_eq!(
			fetch(Path::new("cool"), &paths),
			bpt(&folder, "subdir_2/cool", 1)
		);
		assert_eq!(
			fetch(Path::new("double_subdir/wow.txt"), &paths),
			bpt(&folder, "subdir_2/double_subdir/wow.txt", 1)
		);

		folder.close()?;
		Ok(())
	}
	#[test]
	fn test_fetch_none() -> Result<(), Box<dyn Error>> {
		let folder = tempfile::tempdir()?;

		fs::create_dir(folder.path().join("subdir_1"))?;
		fs::create_dir(folder.path().join("subdir_1/double_subdir"))?;
		fs::create_dir(folder.path().join("subdir_2"))?;

		fs::File::create(folder.path().join("subdir_1/nice.txt"))?;
		fs::File::create(folder.path().join("subdir_2/cool.txt"))?;

		let path1 = folder.path().join("subdir_1");
		let path2 = folder.path().join("subdir_2");
		let paths: Vec<Box<Path>> =
			vec![path1.into_boxed_path(), path2.into_boxed_path()];

		assert_eq!(fetch(Path::new("nothing.txt"), &paths), None);
		assert_eq!(fetch(Path::new("double_subdir"), &paths), None);
		assert_eq!(fetch(Path::new("aaaaa/aaaaa"), &paths), None);

		folder.close()?;
		Ok(())
	}
}
