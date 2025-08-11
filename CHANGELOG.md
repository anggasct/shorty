# Changelog

## [v1.4.0] - 2025-08-11

### Improvements

- **Improvements on interactive mode**
- **Bug fixes and stability improvements**

## [v1.3.0] - 2025-08-09

### Major Release - Complete Feature Overhaul

This release transforms Shorty from a basic alias manager into a comprehensive shell productivity tool with advanced features across multiple domains.

### **New Features**

#### **Backup & Recovery System**

- **Automatic backups** before destructive operations (edit, remove)
- **Named backups** with `--name` option for custom backup names
- **Backup management** with list, restore, and cleanup functionality
- **Timestamped backups** with human-readable metadata
- **Backup cleanup** with `--older-than` option to remove old backups
- Storage location: `~/.shorty/backups/`

#### **Validation & Health Checks**

- **Comprehensive alias validation** with command existence checking
- **Duplicate detection** and resolution
- **Syntax validation** for complex alias formats
- **System command conflict** detection
- **Auto-fix suggestions** with `--fix` flag
- **Security scanning** for dangerous command patterns

#### **Enhanced Search Capabilities**

- **Field-specific search** with `--in` option (command, note, tag)
- **Regex pattern matching** with `--regex` flag
- **Case-insensitive search** by default
- **Smart filtering** that skips empty lines and comments
- **Real-time result counting** and clear no-results messaging

#### **Interactive Terminal UI**

- **Full-screen terminal interface** with keyboard navigation
- **Real-time search and filtering** within the UI
- **Bulk selection** and operations with space bar
- **Visual command preview** with syntax awareness
- **Event-driven architecture** with smooth user experience
- Powered by `ratatui` and `crossterm`

#### **Configuration Management**

- **TOML-based configuration** with `~/.shorty/config.toml`
- **Hierarchical settings** organized by categories:
  - **Backup**: `auto_backup`, `max_backups`, `backup_before_edit`
  - **Display**: `color_output`, `show_line_numbers`, `max_command_length`
  - **Search**: `fuzzy_matching`, `case_sensitive`, `search_in_notes`
  - **Aliases**: `file_path`, `sort_on_add`, `validate_on_add`
- **Dynamic configuration** updates without restart
- **Config validation** and type checking

#### **Statistics & Analytics**

- **Comprehensive analytics** with command analysis and usage patterns
- **Visual ASCII charts** for data visualization
- **Command categorization** (Git, Node.js, Docker, etc.)
- **Tag usage statistics** and popularity metrics
- **Performance insights** and optimization recommendations
- **File metadata** with size, modification time, and health metrics

#### **Data Management**

- **Multi-format export**: JSON, CSV, and Bash script formats
- **Cross-shell import** from `.bashrc`, `.zshrc`, and Fish configurations
- **Import preview** with `--dry-run` option
- **Export summaries** with detailed statistics
- **Data migration tools** for seamless transitions

#### **Template System**

- **Parameterized templates** with `{{variable}}` syntax
- **Template categories** for organized management
- **Parameter validation** and substitution
- **Usage tracking** and statistics
- **Pre-built templates** for common patterns:
  - Docker operations
  - Git workflows
  - SSH tunneling
  - Node.js scripts
- **Template sharing** and distribution capabilities

#### **Category Management**

- **Hierarchical categories** with parent-child relationships
- **Visual icons** and color coding for categories
- **Category-based grouping** and filtering
- **Automatic categorization** suggestions
- **Category statistics** and usage metrics
- **Tree-view display** for category hierarchies

#### **Shell Integration**

- **Tab completion** for bash, zsh, and fish shells
- **Context-aware completion** with alias names and commands
- **Installation status checking** and validation
- **Force reinstall** capability for integration updates
- **Cross-shell compatibility** with automatic detection

#### **Synchronization System**

- **Git-based synchronization** for cross-device alias sharing
- **Device identification** and user tracking with metadata
- **Automatic conflict resolution** with intelligent stashing
- **Remote repository management** with multiple remote support
- **Comprehensive sync status** with ahead/behind tracking
- **Safe sync operations** with backup creation
- **Team collaboration** support for shared alias repositories

#### **Sharing Capabilities**

- **Multiple sharing methods**:
  - **Clipboard**: Direct copy to system clipboard
  - **QR Code**: Generate QR codes for mobile transfer
  - **File Export**: Save as executable shell scripts
- **Cross-platform sharing** with platform-specific optimizations
- **Shareable file formats** for easy distribution

#### **Plugin System**

- **Extensible architecture** with TOML-based plugin manifests
- **Plugin lifecycle management**: install, enable, disable, remove
- **Security validation** and sandboxing for plugin execution
- **Hook system** for extending Shorty functionality
- **Environment variable passing** for plugin context
- **Command validation** and plugin metadata management
- **Future-ready** for plugin marketplace and URL-based installation

### **Enhanced Core Features**

#### **Smart Alias Management**

- **Enhanced add command** with validation and metadata support
- **Improved edit functionality** with backup safety
- **Advanced listing** with filtering and sorting options
- **Safe removal** with confirmation and backup
- **Rich metadata support** for notes and tags

#### **Advanced Search**

- **Multi-field search** across alias names, commands, notes, and tags
- **Pattern matching** with full regex support
- **Search result highlighting** and context display
- **Search history** and saved searches

### **Technical Improvements**

#### **Architecture Enhancements**

- **Modular design** with 16 specialized command modules
- **21 main CLI commands** with extensive subcommand support
- **Robust error handling** with actionable error messages
- **Performance optimization** for handling thousands of aliases
- **Memory efficiency** improvements

#### **New Dependencies**

- **ratatui** (v0.29): Modern terminal UI framework
- **crossterm** (v0.28): Cross-platform terminal manipulation
- **unicode-width** (v0.2): Text width calculations for UI
- **whoami** (v1.4): Device and user identification for sync

#### **Code Quality**

- **Comprehensive testing** across all feature sets
- **Error handling improvements** with detailed error messages
- **Documentation enhancements** with inline comments
- **Performance benchmarking** and optimization

### **Command Reference**

#### **New Commands Added**

- `shorty backup create/restore/list/clean` - Backup management
- `shorty validate [--fix]` - Alias validation and health checks
- `shorty duplicates [--remove]` - Duplicate detection and cleanup
- `shorty interactive` (alias: `shorty i`) - Interactive terminal UI
- `shorty config set/get/list/reset` - Configuration management
- `shorty stats` - Statistics and analytics
- `shorty export --format json|csv|bash` - Data export
- `shorty import --format json|csv|bash` - Data import
- `shorty template add/list/use/remove/show` - Template management
- `shorty category add/list/move/group/show` - Category management
- `shorty install --shell bash|zsh|fish` - Shell integration
- `shorty completion --shell <shell>` - Completion script generation
- `shorty sync init/push/pull/status/reset` - Synchronization
- `shorty sync remote add/list` - Remote repository management
- `shorty share --method clipboard|qr|file` - Sharing capabilities
- `shorty plugin list/install/enable/disable/show/run` - Plugin system

#### **Enhanced Existing Commands**

- `shorty add` - Added validation, backup, and metadata support
- `shorty edit` - Enhanced with backup safety and validation
- `shorty list` - Added filtering, sorting, and display options
- `shorty search` - Complete rewrite with advanced search capabilities
- `shorty remove` - Added backup safety and confirmation

### **Configuration**

#### **New Configuration Files**

- `~/.shorty/config.toml` - Main configuration file
- `~/.shorty/aliases` - Main aliases file (migrated from `~/.shorty_aliases`)
- `~/.shorty/templates.toml` - Template definitions
- `~/.shorty/categories.toml` - Category configurations
- `~/.shorty/plugins.toml` - Plugin registry
- `~/.shorty/sync/` - Synchronization workspace

#### **Configuration Categories**

- **Backup settings** for automatic backup behavior
- **Display preferences** for output formatting and colors
- **Search configuration** for search behavior and preferences
- **Alias management** settings for file paths and validation

### **Performance & Compatibility**

#### **Performance Improvements**

- **Faster startup time** with optimized initialization
- **Memory efficiency** for large alias collections
- **Concurrent operations** where applicable
- **Caching improvements** for frequently accessed data

#### **Compatibility**

- **Cross-platform support**: Linux, macOS, Windows
- **Multi-shell compatibility**: bash, zsh, fish, PowerShell
- **Backward compatibility** with existing alias files
- **Migration tools** for smooth upgrades

### **User Experience**

#### **Enhanced CLI Experience**

- **Emoji indicators** for better visual feedback
- **Color-coded output** with customizable themes
- **Progress indicators** for long-running operations
- **Contextual help** and suggestions
- **Professional error messages** with actionable advice

#### **Interactive Features**

- **Full-screen terminal UI** for visual alias management
- **Keyboard shortcuts** for efficient navigation
- **Real-time feedback** and validation
- **Bulk operations** for managing multiple aliases

### **Developer Experience**

#### **Extensibility**

- **Plugin API** for custom functionality
- **Template system** for common workflow patterns
- **Configuration hooks** for customization
- **Event system** for plugin integration

#### **Documentation**

- **Comprehensive README** with all feature documentation
- **Command reference** with examples and use cases
- **Configuration guide** with all available options
- **Plugin development guide** for extending functionality

### **Migration Guide**

#### **Automatic Migration**

- **Seamless upgrade** from v1.2.0 with automatic data migration
- **Aliases file migration** from `~/.shorty_aliases` to `~/.shorty/aliases` with backup
- **Backup creation** before first migration
- **Configuration initialization** with sensible defaults
- **Validation and cleanup** of existing aliases

#### **New User Setup**

- **Guided installation** with shell integration setup
- **Default configurations** for immediate productivity
- **Example templates** and categories for quick start
- **Shell completion** installation for enhanced experience

### **Breaking Changes**

- **None** - Full backward compatibility maintained
- **New dependencies** required for advanced features
- **Configuration files** created in `~/.shorty/` directory

### **Bug Fixes**

- **Fixed** command parsing for complex alias definitions
- **Resolved** file handling edge cases and permissions
- **Improved** error handling for network operations (sync)
- **Enhanced** performance for large alias collections

### **Acknowledgments**

- **Community feedback** that shaped this major release
- **Contributors** who provided testing and bug reports
- **Open source libraries** that make this functionality possible

---

**Note**: This release represents a complete transformation of Shorty from a simple alias manager to a comprehensive shell productivity suite. All new features have been thoroughly tested and are production-ready.

## [v1.2.0] - 2024-12-10

### Added

- `edit` command to modify existing aliases.
- `uninstall` command to remove the application.

## [v1.1.0] - 2024-12-04

### Added

- `--tag` option for the `add` command to associate tags with aliases.
- `--tag` option for the `list` command to filter aliases by tag.
- `search` command to search for aliases by a keyword.

### Changed

- `add` command now prompts for replacement when an alias already exists, allowing the user to choose whether to replace it.

## [v1.0.0] - 2024-11-30

### Added

- `add` command to add new aliases.
- `list` command to display all aliases.
- `remove` command to remove an alias.
