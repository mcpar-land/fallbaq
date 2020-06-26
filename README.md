# fallbaq

A super simple static file server with a focus on fallback to alternate directories. Written in Rust.

```
fallbaq ./files ./fallback_files /more_fallback_files
```

# Installation

Install with [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

```
cargo install fallbaq
```

Alternatively, download one of the [releases](https://github.com/mcpar-land/fallbaq/releases)

# Use

Pass a number of directories. Files will be loaded starting from the first, and then from following directories if they aren't present. A 404 will be returned if the file is not found in any directory.

Think of it as layering different folders on top of each other.

Change the port:

```
fallbaq ./files -p 1337
fallbaq ./files --port 1337
```

# Example:

Consider a file system like this:

```markdown
# ./files

- one.png
- two.png
- bars
  - bar1.png
  - bar2.png

# ./fallback_files

- one.png
- two.png
- three.png
- bars
  - bar3.png
  - foos
    - foo1.png

# /var/www/html

- four.png
- bars
  - bar1_old.png
  - bar2_old.png
```

Running the following command:

```
fallbaq ./files ./fallback_files /var/www/html
```

Will expose a file system that looks like this to requests:

<!-- prettier-ignore-start -->
```markdown
- one.png             (from ./files)
- two.png             (from ./files)
- three.png           (from ./fallback_files)
- four.png            (from /var/www/html)
- bars                
  - bar1.png          (from ./files)
  - bar1_old.png      (from /var/www/html)
  - bar2.png          (from ./files)
  - bar2_old.png      (from /var/www/html)
  - foos
    - foo1.png        (from ./fallback_files)
```
<!-- prettier-ignore-end -->

# Future Features

- Serving as a proxy to another http location
- fallback to a single file
- some sort of `.fallbaqignore` file you can include in your directories.
