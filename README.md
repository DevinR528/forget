# Forget
## Customizable, Simple, Cross-platform Terminal "todo" app.

[![Build Status](https://travis-ci.com/DevinR528/forget.svg?branch=master)](https://travis-ci.com/DevinR528/forget)
[![Latest Version](https://img.shields.io/crates/v/forget.svg)](https://crates.io/crates/forget)

A beautiful (thanks to [tui.rs]()) command line app to keep you from forgetting!! `forget`
uses [termion]() for rendering as such it is cross platform (fonts may differ). `forget` is __highly__
customizable, key bindings, colors, icons, text and titles. `forget`'s configuration and "database" files will save
to `$HOME/.forget/[file].json` this is where the config file can be edited. Along with each sticky note item
a command can be run when the item is selected (press Enter while highlighted) this will run in a separate
process and will not affect the UI. `forget` is a multi-threaded application the UI event loop and input loop
each run on a separate thread as well as any command spawned, everything is cleaned up when the main thread exits.

![forget-demo](https://github.com/DevinR528/forget/resources/forget-demo.gif)

# Install
```bash
cargo install forget
```

# Run
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
 * **ctrl-h**
    - add new sticky note.
 * **ctrl-n**
    - add new todo item to current sticky note.
 * **ctrl-e**
    - edit currently selected todo item of current sticky note.
 * **ctrl-k**
    - add new note to current sticky note.
 * **ctrl-u**
    - removes current sticky note.
 * **ctrl-s**
    - save everything to "data base".

# Customize
Everything is customizable with the `./.forget/config.json` file unfortunately spelling
and capitalization matter. 
Note: Ctrl-j, Ctrl-i and Ctrl-m are all highjacked by bash to be output as different characters
DO NOT USE THEM FOR KEY MAPPINGS.
```json
{
  "title": "Forget It",
  "new_sticky_note_char_ctrl": "h",
  "new_note_char_ctrl": "k",
  "new_todo_char_ctrl": "n",
  "edit_todo_char_ctrl": "e",
  "mark_done": "Backspace",
  "remove_todo": "Delete",
  "remove_sticky_note_char_ctrl": "u",
  "save_state_to_db_char_ctrl": "s",
  "exit_key_char_ctrl": "q",
  "highlight_string": "✔️",
  "app_colors": {
    "normal": {
      "fg": "White",
      "bg": "Reset",
      "modifier": "RESET"
    },
    "highlight": {
      "fg": "Yellow",
      "bg": "Reset",
      "modifier": "BOLD"
    },
    "tabs": {
      "fg": "Cyan",
      "bg": "Reset",
      "modifier": "BOLD"
    },
    "titles": {
      "fg": "Red",
      "bg": "Reset",
      "modifier": "BOLD"
    }
  }
}
```
Options are listed below.

# Options
When changing any of these care must be taken to match capitalization and spelling.

### Character
All Valid Utf-8 single character byte sequences.

### Keys, not characters or strings
Backspace
Left
Right
Up
Down
Home
End
PageUp
PageDown
BackTab
Delete
Insert
F(u8)
Null
Esc

### Colors
Reset,
Black,
Red,
Green,
Yellow,
Blue,
Magenta,
Cyan,
Gray,
DarkGray,
LightRed,
LightGreen,
LightYellow,
LightBlue,
LightMagenta,
LightCyan,
White,
Rgb(u8, u8, u8),
Indexed(u8),

### Text Modifiers
BOLD
DIM
ITALIC
UNDERLINED
SLOW_BLINK
RAPID_BLINK
REVERSED
HIDDEN
CROSSED_OUT
RESET

### Titles and Icons
Any valid Utf-8 characters will work.


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
