# Sway status bar
This is a custom status bar using the swaybar protocol, each block is an async function. 
Because of how the swaybar protocol works, every block update forces all blocks to be updated,
but using async means that less work is wasted on the bar side of things.

The code in [main.rs](./main.rs) should be fairly self explanatory if you want to modify this to your liking.

# Installation

Run
```sh
cargo install --path .
```
then in your sway bar config, set:
```
    swaybar_command sway_status_bar
```

---

Issues and PRs welcome.
