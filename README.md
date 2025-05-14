# rpgmv-tool
A tool with utilities for interacting with RPGMaker MV assets.
Also see [rpgmv-image-viewer](rpgmv-image-viewer/README.md) for a graphical application to view encrypted images.

## Subcommands

### decrypt
WIP, will also likely be renamed.

### commands2py
This is a tool that can convert compiled event command JSON into Python for easier readability.
Note that the generated Python is not intended to be run and will fail if you do so.
The generated code will reference functions and variables that don't exist,
and may not even be syntactically valid.
The generated Python is purely for inspection.
The simplest way to use this command is like so:
```bash
rpgmv-tool commands2py -i <path-to-json> --id <item-id>
```

#### Arguments
`-i / --input`: The path to the input JSON file. This is required.  
`-o / --output`: The path to the output file. This is optional. It defaults to `./out.py`.  
`-c / --config`: The path to the config file. This is optional.  
`--id`: The id of the item to convert. This is required.  
`--event-page`: The page of the event to extract. This is required iff the input file is a Map or Troop.  

#### Config
This command supports a config file to change the output.
All tables are optional.
It has the following format:
```toml
# A mapping of switch ids to names.
# The command will use the given name instead of generating a name.
[switches]
# 2 = "cool_switch_name"

# A mapping of variable ids to names.
# The command will use the given name instead of generating a name.
[variables]
# 42 = "the_answer"

# A mapping of common event ids to names.
# The command will use the given name instead of generating a name.
[common-events]
# 5 = "do_it"

# A mapping of actor ids to names.
# The command will use the given name instead of generating a name.
[actors]
# 1 = "main_character_actor"

# A mapping of skill ids to names.
# The command will use the given name instead of generating a name.
[skills]
# 1 = "attack_skill"

# A mapping of item ids to names.
# The command will use the given name instead of generating a name.
[items]
# 123 = "health_potion"

# A mapping of state ids to names.
# The command will use the given name instead of generating a name.
[states]
# 43 = "blind_state"

# A mapping of troop ids to names.
# The command will use the given name instead of generating a name.
[troops]
# 86 = "enemy_force"

# A mapping of armor ids to names.
# The command will use the given name instead of generating a name.
[armors]
# 32 = "the_best_armor"
```

### encrypt-png
This is a tool that can encrypt pngs into the "rpgmvp" format.
It can be used like so:
```bash
rpgmv-tool encrypt-png -i <path/to/png/file.png> -o <path/to/new/file.rpgmvp> -k <key as hex>
```

## License
Licensed under either of
 * Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)
at your option.

## Contributing
Unless you explicitly state otherwise, 
any contribution intentionally submitted for inclusion in the work by you, 
as defined in the Apache-2.0 license, 
shall be dual licensed as above, 
without any additional terms or conditions.