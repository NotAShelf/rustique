# Lithic – English help strings
# Migrated from en_help.toml

## Main subcommands
cmd-config   = Manage config options for Lithic.
cmd-delete   = Remove mods and backups.
cmd-download = Download a Vintage Story executable.
cmd-help     = Print this message or the help of the given subcommand(s)
cmd-info     = Get more information about the mod specified.
cmd-install  = Install a specific mod. Must use the mod_id, Example: ./Lithic install alchemy
cmd-list     = List installed mods and their versions and any missing dependencies. Running sync first will show any available updates to your mods.
cmd-misc     = Miscellaneous items for Lithic, like shell auto-completion and 1-click mod installation.
cmd-modpack  = Create, download, update modpacks for VintageStory.
cmd-search   = Search the mod website for new mods, Example: ./Lithic search -q magic
cmd-self     = Manage the Lithic binary; Check for updates, perform updates.
cmd-sync     = Checks with the VintageStory mods website for any updates to mods you have installed. Run update after this command to update your mods.
cmd-update   = Updates a specific mod OR all mods installed. Runs sync after completion.

## sync subcommand flags
sync-game-versions = Sync a local list of all game versions from the api. This is used with version pinning to ensure we have accurate version numbers.
sync-search-db     = Sync the mod info from the api endpoint /api/mods and save it locally.

## verbose flag
flag-verbose = Shows info level logging messages. This is very noisy, used for debugging purposes.
