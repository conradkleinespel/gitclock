# GitClock

Want all your commits to have dates within specific ranges? Want to avoid leaking your current timezone through your commit log?

That's what GitClock does for you.

> [!NOTE]
> This project was initially developed in Typescript. I've now rewritten it in Rust. The new version comes with improved
> performance, bugfixes and out-of-the-box NixOS support.

![](gitclock.png)

## Installation

To install GitClock, you can run the following:

```shell
cargo install --git https://github.com/conradkleinespel/gitclock
```

There is also first-class NixOS support through `default.nix`:

```nix
{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    (callPackage (fetchFromGitHub {
      owner  = "conradkleinespel";
      repo   = "gitclock";
      rev    = "git-commit-sha-here";
      sha256 = "nix-sha256-here";
    }) {})
  ];
}
```

## Usage

```shell
# Set your public schedule
# For example, 9am to 5pm on week days
gitclock timeslot --add --days "1-5" --start "0900" --end "1700"

# Set your public timezone to avoid leaking it during travel
# Available formats: https://en.wikipedia.org/wiki/List_of_tz_database_time_zones
gitclock config --timezone Europe/Paris

# Commit with the next available date in your timeslots (±15 minutes)
# Any options from "git commit" will work
gitclock commit -m "My commit message"

# Rebase with commit dates within timeslots
# Any options from "git rebase" will work
gitclock rebase -i <commit-sha> 

# Push commits whose date is in the past
# Any options from "git push" will work
gitclock push

# Rewrite history of your existing git repository so that all commits
# get a date within your schedule and timezone
# /!\ Do this in a separate branch, just in case you're unhappy with the result
gitclock rewrite-history

# Configure git hooks to prevent accidental misuse of `git commit/push/rebase`
echo "gitclock pre-commit-hook" >> .git/pre-commit
chmod +x .git/pre-commit
echo "gitclock pre-push-hook" >> .git/pre-push
chmod +x .git/pre-push
echo "gitclock pre-rebase-hook" >> .git/pre-rebase
chmod +x .git/pre-rebase
```

## Development

Unit and integration tests run under timezone `Africa/Nairobi`, because that is a timezone without summer/winter time, which helps keep tests deterministic throughout the year.

## License

The source code is released under the Apache 2.0 license.
