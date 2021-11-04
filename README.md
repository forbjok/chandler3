# Chandler 3

## Introduction
See a thread you consider worthy of archival?
Don't trust the 4chan archives to remain available?
Or perhaps it's not on 4chan, but on some tiny obscure imageboard nobody else has heard of or considered worth archiving?

Chandler to the rescue!

**Chandler 3** is the successor to [Chandler 2](https://github.com/forbjok/chandler2), rewritten from scratch in Rust.

For sites where this is supported (currently 4chan and Tinyboard-compatible sites), posts from previous threads which are later deleted will be preserved when updating threads.

Supported platforms are **GNU/Linux** (tested on Arch Linux, but should work others as well) and **Microsoft Windows**, but it might work on other *nixes as well if it's possible to get the Rust compiler working on them.

## Installing
Coming soon...

For GNU/Linux, see **Compiling**.

## Compiling
1. Install Rust using the instructions [here](https://www.rust-lang.org/tools/install) or your distro's package manager.
2. Clone this repository and execute the following command in it:
```
$ cargo build --release
```

Voila! You should now have a usable executable in the `target/release` subdirectory.

## How to use
To download a thread once and exit:
```
$ chandler grab <thread url>
```

To watch and update a thread indefinitely:
```
$ chandler watch <thread url>
```

That's the basics. For more parameters, see
```
$ chandler --help
```

## Where do threads go?
By default, threads are saved to **~/Downloads/chandler3** on GNU/Linux and **Downloads\chandler3** on Windows.

This can be overridden using the **download-path** setting in your **config.toml** configuration file.

## Configuration
There are 3 configuration files used by Chandler:
* config.toml
* sites.toml
* cli.toml

These files will by default be read from the default configuration directory.
The default configuration directory is **~/.config/chandler3** on GNU/Linux or at **%APPDATA%\chandler3** on Windows.

The following command can be used to generate a set of default configuration files in the default location:
```
$ chandler generate-config
```

## Chandler configuration: config.toml
This is the main Chandler configuration file.
It can be used to customize the download path.

Example:
```toml
# Specify path to download threads to.
# Subdirectories will automatically be created for each site, board, thread, etc.
download-path = "/PATH/TO/DOWNLOADS"
```

## Site configuration: sites.toml
The **sites.toml** file is entirely optional.
It is not needed if the site you intend to use is one supported out of the box, such as 4chan.
It can however be useful if you want to use it with other more obscure boards that use an imageboard software supported by one of the parsers.

By default, unknown sites will automatically use the **basic** parser and a very generic URL regex that _should_ work with most imageboards.

Example:
```toml
# Include built-in sites.
include-builtin-sites = true

[sites."examplechan"]
url-regexes = ['^http(?:s)?://examplechan.org/(.+)/res/(\d+)']
parser = "tinyboard"
```

A custom site can be specified as shown above.

The capture groups in the URL regex will determine the directory structure that gets created for each thread inside your **download path**.
One subdirectory for each capture group.

## CLI configuration: cli.toml
This one is as optional as it gets.
It allows you to customize some mostly cosmetic details in the CLI, such as whether to display progress and what style of progress bar to use.
That's pretty much it.

Example:
```toml
# Progress display options.
[progress]
enable = true
bar-style = "dot"
```

Supported bar-styles are **dot** (the default), **hash** and **arrow**.

## Parsers
These are the possible values for the **parser** property of a **site**.

Currently, the following parsers exist:
* **fourchan**
* **tinyboard**
* **basic**
* **aspnetchan**
* **foolfuuka**
* **kusabax**
* **lainchan**
* **ponychan**

**fourchan**, as the name suggests is specifically for 4chan. For all I know, there may be other imageboards that happen to use the exact same HTML layout (class names, etc) as 4chan, and thus work with this parser. However, don't count on this working with anything other than 4chan.

**tinyboard** is for Tinyboard-compatible sites. I say "compatible", because not all sites that are based on Tinyboard are necessarily Tinyboard-compatible - they could have customized their HTML layouts and changed class names, etc. so that this parser will no longer work with them. For most Tinyboard-based sites, this should work though.

**basic** is completely basic. It does not rely on anything HTML-layout specific at all, and does not support preserving deleted posts. It should work with **ANY** site.

Most of the others should be fairly self-explanatory, being named after either the site or imageboard software they were written for.
As with the aforementioned, it's possible that they may coincidentally work with other sites or imageboard softwares, and it's also possible that they may not work with certain sites using them if they have customized their HTML class names or other details.

## Chandler projects
A chandler "project" (I use this term for lack of a better one) is what gets created when Chandler is used to download a thread.

In addition to the **thread.html** and any files downloaded from the thread, each project will contain a **.chandler** directory.

It contains the following:
* Various metadata, such as the thread's URL
* State related to downloading the thread
* All original pristine HTMLs as downloaded directly from the server

If you are 100% sure you are done downloading/updating a thread, you can safely delete this, however I would personally recommend keeping it.

## Rebuilding a project
Since a project stores all the original HTMLs, it is possible to completely rebuild the **thread.html** from the original HTMLs.

This is done using the `$ chandler rebuild <project path>` command.

Possible reasons for doing this:
* Your **thread.html** somehow got corrupted or deleted
* A bug in Chandler caused **thread.html** to have an error, and an updated version has come out that fixes the bug
* You downloaded a thread using the **basic** parser, which does not preserve deleted posts, but a new parser was later added that supports the site it was downloaded from and now you want those deleted posts back in your **thread.html**

For this reason, I recommend keeping the **.chandler** directory.
