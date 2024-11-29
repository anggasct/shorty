
# Shorty

Shorty is a simple command-line tool to manage shell aliases. You can add, list, and remove aliases easily, while keeping them organized in a file named `.shorty_aliases` in your home directory.


## Features

- **Add Alias**: Create new aliases with optional notes.
- **List Aliases**: View all current aliases stored in `.shorty_aliases`.
- **Remove Alias**: Delete an existing alias from the list.


## Installation

To install `shorty`, simply run the following command:


```bash
curl -sSfL https://github.com/anggasct/shorty/raw/main/install.sh | sh
```

This script will automatically download and install the appropriate binary for your operating system (Linux or macOS). It will also add the necessary configuration to your shell to ensure aliases are applied.
    
## Usage

Once installed, you can use `shorty` from the command line.

### Add New Alias
To add new alias:

```bash
shorty add <alias> <command> [--note <note>]
```

- `<alias>`         : The alias name you want to create.
- `<command>`       : The command that the alias should execute.
- `--note <note>`   : (Optional) Add a note to your alias.

Example:

```bash
shorty add gs "git status" --note "Alias for git status"
```

This will add the alias `gs` for `git status` with a note "Alias for git status" to the `.shorty_aliases` file.


### List All Aliases
To list all the aliases stored in `.shorty_aliases`:

```bash
shorty list
```

This will print all the aliases currently saved in the file.


### Remove an Alias
To remove an alias

```bash
shorty remove <alias>
```

- `<alias>`: The alias you want to remove.

Example:
```bash
shorty remove gs
```

This will remove the alias gs from the `.shorty_aliases` file.
