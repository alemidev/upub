# upub cli

command line interface tools for `upub`

everything is pretty well documented: just add `--help` to get detailed info


```sh
$ upub --help
micro social network, federated

Usage: upub [OPTIONS] <COMMAND>

Commands:
  config    print current or default configuration
  migrate   apply database migrations
  cli       run maintenance CLI tasks
  monolith  start both api routes and background workers
  serve     start api routes server
  work      start background job worker
  help      Print this message or the help of the given subcommand(s)

Options:
  -c, --config <CONFIG>    path to config file, leave empty to not use any
      --db <DATABASE>      database connection uri, overrides config value
      --domain <DOMAIN>    instance base domain, for AP ids, overrides config value
      --debug              run with debug level tracing
      --threads <THREADS>  force set number of worker threads for async runtime, defaults to number of cores
  -h, --help               Print help
```

---

```sh
$ upub cli --help
run maintenance CLI tasks

Usage: upub cli <COMMAND>

Commands:
  faker           generate fake user, note and activity
  fetch           fetch a single AP object
  relay           act on remote relay actors at instance level
  count           recount object statistics
  update          update remote actors
  register        register a new local user
  nuke            break all user relations so that instance can be shut down
  thread          attempt to fix broken threads and completely gather their context
  cloak           replaces all attachment urls with proxied local versions (only useful for old instances)
  fix-activities  restore activities links, only needed for very old installs
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```
