# Age of Calamity Mod Merger

This is a fork of [Raytwo's rdb_tool](https://github.com/Raytwo/rdb_tool). It was rewritten as an attempt to create an AOC mod merging tool. As of today (25.05.2024), only texture mods are available, so only those were tested.

# Setup and Usage

1. Follow [this tutorial](https://gamebanana.com/tuts/17528#H1_0) to dump your OWN copy of Age of Calamity.
2. Go to the AOC mods directory.
3. Copy the mods to be merged there.
4. Copy the `AOC_mods_merger.exe` there as well.
5. Either double-click on the exe or open the AOC mods directory in the cmd/powershell terminal and run it with the command:

    ```
    AOC_mods_merger.exe <mods_directory>
    ```

    If no `mods_directory` argument is provided, the merger will try to work in the current directory.

6. A window should pop up asking for the AOC romfs dump directory. Select it and press OK (this needs to be done only once).
7. If the program is run for the first time, it will cache some AOC info from the user's dump - this will take less than 10 seconds and will be performed only once.
8. If the command succeeds, a new folder should be created - `000_AOC_MERGED_MODS`.
9. Run the game and test if everything works.

# Credits

- [Raytwo](https://github.com/Raytwo) - original code of rdb_tool
