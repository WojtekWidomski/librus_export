# LIBRUS Synergia Message Export

This program allows to download all messages from your Librus Synergia account and save them in JSON format. (Librus Synergia is electronic grade book system used in many schools in Poland.)

## Features
- Export from all folders, including Archive
- Save big groups of message receivers to separate file
- Easy CLI (Command line interface) with progressbars from `indicatif` library.
- Written in Rust

## Output:
Exported data are saved in folder named `export_FirstName_LastName`.  
Following files are created:

- `messages_folder_name.json` (for each messages folder) - all messages from folder
- `groups.json` - Groups of receivers

## Usage
Clone this repository and run this program using this command:

```
cargo run --release
```

The program will ask, how many users should be considered as big group. Small group of receivers will be saved in the same file as messages. Big groups will be saved in array in separate file and only index in this array will be saved if file with messages. Default is 10.

This is useful, because some teachers very often send messages to all students in the school.

Then the program will ask for username and password. Then you will have to choose, which folder to download. By default all folders are selected.

## Used technologies
- Rust programming language
- Please see [Cargo.toml](Cargo.toml) for used crates
- Altough [this repository](https://github.com/kbaraniak/librus-api/) is not a dependency of this program, I used it as a refecence, when implementing authentication.
