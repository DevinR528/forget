# Forget
## Customizable, Simple, Cross-platform Terminal "todo" app.

[![Build Status](https://travis-ci.com/DevinR528/forget.svg?branch=master)](https://travis-ci.com/DevinR528/forget)
[![Latest Version](https://img.shields.io/crates/v/forget.svg)](https://crates.io/crates/forget)

A beautiful (thanks to [tui.rs]()) command line app to keep you from forgetting!! `forget`
uses [termion]() for rendering as such it is cross platform (fonts may differ). 

# Install
```bash
cargo install forget
```

## Use
In order to navigate around `forget`:
 * **up arrow & down arrow**
    - selects item or question.
 * **left arrow & right arrow**
    - selects "tab" or Sticky Note.
 * **backspace**
    - cross an item off without removing it.
 * **delete**
    - remove an item.
 * **ctrl-s**
    - add new sticky note.
 * **ctrl-t**
    - add new todo item to current sticky note.
 * **ctrl-n**
    - add new note to current sticky note.
 * **ctrl-k**
    - removes current sticky note.
 * **ctrl-p**
    - save everything to "data base".

# Customize
Everything is customizable with the `./.forget/config.json` file unfortunately spelling
and capitalization matter.
```json
{
  "title": "Forget It",
  "new_sticky_note_ctrl": "s",
  "new_note_ctrl": "n",
  "new_todo_ctrl": "t",
  "mark_done": "Backspace",
  "remove_todo": "Delete",
  "remove_sticky_note_ctrl": "k",
  "save_state_to_db_ctrl": "p",
  "exit_key_ctrl": "q",
  "highlight_string": "✏️",
  "app_colors": {
    "normal": {
      "fg": "Gray",
      "bg": "Reset",
      "modifier": "RESET"
    },
    "highlight": {
      "fg": "Yellow",
      "bg": "Reset",
      "modifier": "ITALIC"
    },
    "tabs": {
      "fg": "Cyan",
      "bg": "Reset",
      "modifier": "BOLD"
    },
    "titles": {
      "fg": "Magenta",
      "bg": "Reset",
      "modifier": "BOLD"
    }
  }
}
```
Options are listed below.

# Run
```bash
cargo install forget
```

# Examples


#### License
<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
</sub>
