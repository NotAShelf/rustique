

# Version 0.2.0-alpha
* Fixed api error message with blank info. A proper message is displayed when a mod has an empty mod_id.
* Config file is now live! You can easily set the default mod directory so you don't have to use -m for each use if different than default.
* To manage the config file you use Rustique directly. Checkout `Rustique help config` for all options. Note that not everything is implemented yet.
* Reorganized code base a bit, this doesn't affect anything user side, but it's a win for me. :3
* Added logging lib and implemented --verbose and --debug. --verbose will show some extra messages if you notice some problems. --debug you should only use if told to do so, its extremely noisy and floods the terminal.
* The description from the mod files is now sanitized to strip any newline or tab characters as it messes with the `list` table formatting. If you find any other mods that don't show up correctly, please report it.
* Fixed some versioning bugs when using sync and update that would cause some mods to not be updated. 
* Added an operation time footer for `list`, `update`, `sync`, and `install`. This can be turned off in the configs.
* List shows total mods installed at the bottom. For now, this only shows the valid mods that Rustique can actually manage. Any non-zip mods are ones that list can't read, will not be counted. 
* The list table style is slightly more compact now and the dependencies lists no longer wrap in the middle of a long mod_id.
* Adjusted the look'n feel of the tables.
* Information text has a border now.
* Rustique no longer deletes mods that are malformed during the update command, it reports the problem but leaves it alone.
* Full rework of mod installation and dependency resolution. Update & Install are dramatically faster.