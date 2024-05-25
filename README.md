# Age of Calamity mod merger

This is a fork of [Raytwo's rdb_tool](https://github.com/Raytwo/rdb_tool). It was rewritten as an attempt to create AOC mod merging tool. As of today (25.05.2024) only textures mods are available, so only those were tested.

# Setup and usage

1. Follow [this](https://gamebanana.com/tuts/17528#H1_0) tutorial to dump your OWN copy of Age of Calamity
2. Go to AOC mods directory
3. Copy the mods to merge there.
4. Copy the `AOC_mods_merger.exe` there as well.
5. Either double click on the exe or open AOC mods directory in cmd/powershell terminal and run it by command:

```
AOC_mods_merger.exe <mods_directory>
```
If no `mods_directory` argument is provided then merger will try to work in current directory.

6. The window should pop up asking for AOC romfs dump directory, select it and press OK (needs to be done only once).
7. If the program is run for the first time it will cache some AOC info from user's dump - it will take less than 10s and will be performed only once.
8. If command succeeds, new folder should be created - `000_AOC_MERGED_MODS`
9. Run the game and test if everything works

# Credits

- [Raytwo](https://github.com/Raytwo) - original code of rdb_tool