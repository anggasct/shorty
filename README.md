# Shorty

Shorty is a simple command-line tool to manage shell aliases. You can add, list, search, edit, and remove aliases easily, while keeping them organized in a file named `.shorty_aliases` in your home directory.

## Features

- **Add Alias**: Create new aliases with optional notes.
- **List Aliases**: View all current aliases.
- **Search Alias**: Search for aliases.
- **Edit Alias**: Modify an existing alias.
- **Remove Alias**: Delete an existing alias from the list.

## Installation

To install `shorty`, run the following command:

```bash
curl -sSfL https://github.com/anggasct/shorty/raw/main/install.sh | sh
```

This script will download and install the appropriate binary for your operating system (Linux or macOS) and add the necessary configuration to your shell to ensure aliases are applied.

## Usage

Once installed, you can use `shorty` from the command line.

### Add New Alias

To add a new alias:

```bash
shorty add <alias> <command> [--note <note>] [--tags <tags>]
```

- `<alias>`: The alias name you want to create.
- `<command>`: The command that the alias should execute.
- `--note <note>`: (Optional) Add a note to your alias.
- `--tags <tags>`: (Optional) Add a tags to your alias.

Example:

```bash
shorty add gs "git status" --note "Alias for git status"
```

This will add the alias `gs` for `git status` with a note "Alias for git status" to the `.shorty_aliases` file.

### List All Aliases

To list all the aliases stored in `.shorty_aliases`:

```bash
shorty list [--tags <tags>]
```

- `--tags <tags>`: (Optional) Filter aliases by tags.

This will print all the aliases currently saved in the file.

### Search for an Alias

To search for an alias:

```bash
shorty search <alias>
```

- `<alias>`: The alias you want to search.

Example:

```bash
shorty search gs
```

This will search for the alias `gs` in the `.shorty_aliases` file.

### Edit an Alias

To edit an existing alias:

```bash
shorty edit <alias> <new_command> [--note <new_note>] [--tags <new_tags>]
```

- `<alias>`: The alias you want to edit.
- `<new_command>`: The new command that the alias should execute.
- `--note <new_note>`: (Optional) Add or update the note for your alias.
- `--tags <new_tags>`: (Optional) Add or update the tags for your alias.

Example:

```bash
shorty edit gs "git status -sb" --note "Updated alias for git status"
```

This will update the alias `gs` to execute `git status -sb` with an updated note "Updated alias for git status" in the `.shorty_aliases` file.

### Remove an Alias

To remove an alias:

```bash
shorty remove <alias>
```

- `<alias>`: The alias you want to remove.

Example:

```bash
shorty remove gs
```

This will remove the alias `gs` from the `.shorty_aliases` file.

## Contributing

Contributions are welcome! Please fork the repository and submit a pull request with your changes.
