# v0.2.1

- Fixed binary name, it will now be `dots` instead of `dots-bin`

# v0.2.0

- Add a verbosity flag (`-q...` and `-v...`) to control logging level (default: INFO)
- Instead of a `{home}/foo`, tilde is expanded now: `~/foo`
- Renamed variables `config` and `cache` to `config_dir` and `state_dir` in interpolations
- Add support for interpolation of environment variables, i.e. `{$HOME}`

# v0.1.0

Initial release
