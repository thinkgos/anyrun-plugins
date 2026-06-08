# Menu Bar

A two-level menu bar plugin for [anyrun](https://github.com/anyrun-org/anyrun). Define custom menus with items and launch commands via a simple prefix-based interface.

## Usage

Type the prefix to list menus, then use the delimiter to drill into a specific menu and search its items:

```
:m                        # List all menus
:m a                      # Open menu with key 'a'
:m a/Color                # Fuzzy search items in menu 'a'
```

## Configuration

```ron
// <Anyrun config dir>/menu-bar.ron
Config(
  // The prefix that triggers the plugin
  prefix: ":m",
  // The delimiter to select the menu item
  delimiter: '/',
  // Maximum number of entries to return
  max_entries: 10,
  // The terminal used for running terminal based menu items
  // If left as None, the plugin auto-detects from:
  // alacritty, kitty, ghostty, wezterm, foot, wterm
  terminal: Some(Terminal(
    // The main terminal command
    command: "alacritty",
    // What arguments should be passed to the terminal process to run the command correctly
    // {} is replaced with the command in the desktop entry
    args: "-e {}",
  )),
  // The menus defined in the plugin
  menus: [
    Menu(
      title: "Dev Tools",
      key: 'd',
      description: Some("Development utilities"),
      children: [
        MenuItem(
          title: "Color Picker",
          description: Some("Pick a color from screen"),
          exec: "echo 'pick a color'",
          // Set to true to run in terminal
          // term: false,
        ),
        MenuItem(
          title: "Top",
          description: Some("System monitor"),
          exec: "top",
          term: true,
        ),
      ],
    ),
  ],
)
```

### Menu fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `title` | `String` | required | Display name of the menu |
| `key` | `char` | required | Unique identifier for the menu |
| `description` | `Option<String>` | `None` | Optional description shown alongside the title |
| `children` | `Vec<MenuItem>` | `[]` | List of menu items |

### MenuItem fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `title` | `String` | required | Display name of the item |
| `description` | `Option<String>` | `None` | Optional description shown alongside the title |
| `exec` | `String` | required | Shell command to execute when selected |
| `term` | `bool` | `false` | Whether to run the command in a terminal |
