# dots

A cozy cross-platform dotfiles manager.

## Copying files

With the following `~/dots.toml`:

```toml
[[dir]]
input = "my_configs"
output = "{config}/foo"
```

All files within `~/my_configs` (recursively) will be copied to the `{config}/foo` directory, at the same location.

So a file `~/my_configs/foo/bar.txt` will be copied to `~/.config/foo/bar.txt` on Linux (on Windows and MacOS it will use the platform's respective directory)

`{config}` expands to the appropriate config directory on your platform. These are the available expansions:

- `{config}`: Config directory
- `{home}`: Home directory
- `{cache}`: Cache directory
- `{state_dir}`: State directory

## Granular control for each file

You can control where each file will be copied by adding a single line at the top of a file. So if `configs/glazewm.yaml`'s first line is this:

```
@dots --path "{home}/.glzr/glazewm/config.yaml"
```

It will copy the file to the appropriate location in the home folder, instead of copying it in the config directory.

As long as the first line *contains* `@dots ...` then it will work.

## Links

You can put links into your `dots.toml`:

```toml
[[link]]
url = "https://raw.githubusercontent.com/catppuccin/nushell/05987d258cb765a881ee1f2f2b65276c8b379658/themes/catppuccin_mocha.nu"
path = "my_configs/nushell/catppuccin.nu"
sha256 = "d639441cd3b4afe1d05157da64c0564c160ce843182dfe9043f76d56ef2c9cdf"
```

That will download file at the specified `url` into `~/my_configs/nushell/catppuccin.nu`.

A `sha256` can be *optionally* provided for security. If the file at that location's sha256 does not match the provided sha256, it will **not** be downloaded.

## Templating

Each file in any `input` directory in `[[dir]]` has full support of the [handlebars](https://handlebarsjs.com/) templating language. One use case of this is to avoid duplicating the same content in a single file.

Say you have the following `~/my_configs/helix/config.toml`:

```toml
[keys.normal]
# DEFAULT: A-C
A-c = "copy_selection_on_prev_line"
# DEFAULT: A-J
C-j = "join_selections_space"

[keys.select]
# DEFAULT: A-C
A-c = "copy_selection_on_prev_line"
# DEFAULT: A-J
C-j = "join_selections_space"
```

There is duplication. In order to avoid this, you can do this:

```toml
{{#* inline "rebindings" }}
# DEFAULT: A-C
A-c = "copy_selection_on_prev_line"
# DEFAULT: A-J
C-j = "join_selections_space"
{{/inline}}

[keys.normal]
{{> rebindings }}

[keys.select]
{{> rebindings }}
```

All instances of `{{> rebindings }}` will be replaced by the `inline` block. This is just one of many features that a templating language provides
