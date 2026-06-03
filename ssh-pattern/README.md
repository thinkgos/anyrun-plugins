# SSH Pattern

Search and connect to SSH hosts defined in your SSH config files.

## Usage

Type in the search bar to fuzzy-match against your SSH hosts:

```
/ <search>              # Search hosts by name
```

Select a specific terminal emulator:

```
/><terminal_key or command> <search>   # Use terminal to search hosts
/>a myhost                  # Use alacritty to connect to myhost
/>alacritty myhost          # Use alacritty to connect to myhost
/>k myhost                  # Use kitty to connect to myhost
/>kitty myhost              # Use kitty to connect to myhost
```

Built-in terminal keys:

- `a` - `alacritty`
- `k` - `kitty`
- `g` - `ghostty`
- `we` - `wezterm`
- `f` - `foot`
- `wt` - `wterm`

## Configuration

```ron
// <Anyrun config dir>/ssh-pattern.ron
Config(
  // The prefix that triggers the plugin
  prefix: "/",
  // The delimiter to select the terminal.
  terminal_delimiter: '>',
  // The default terminal to use for SSH commands
  // Leave as None to use first available terminal
  terminal: None,
 // Additional custom terminals
  terminals: [
  //   Terminal(
  //     key: "a",
  //     command: "alacritty",
  //     args: "-e {}",
  //   ),
  ],
  // Maximum number of entries to return
  max_entries: 6,
  // Paths to SSH config files
  ssh_config_paths: ["~/.ssh/config"],
)
```
