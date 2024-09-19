#!/usr/bin/env node

const process = require("node:process");
const { Command } = require("commander");
const { Config } = require("./src/config");
const { info } = require("./src/commands/info");
const { configure } = require("./src/commands/configure");
const { timeslot } = require("./src/commands/timeslot");
const { commit } = require("./src/commands/commit");
const { push } = require("./src/commands/push");
const { preCommitHook } = require("./src/commands/preCommitHook");
const { prePushHook } = require("./src/commands/prePushHook");
const console = require("node:console");
const { rewriteHistory } = require("./src/commands/rewriteHistory");
const { rebase } = require("./src/commands/rebase");
const { preRebaseHook } = require("./src/commands/preRebaseHook");

function main() {
  const config = Config.createFromConf();
  const program = new Command();

  try {
    config.checkConfig();
  } catch (err) {
    console.error(`Configuration error: ${err.message}`);

    console.log("");
    console.log("To fix your configuration, edit:");
    console.log(`  ${config.getFilePath()}`);

    process.exit(2);
  }

  program
    .name("gitclock")
    .description("A CLI to schedule Git commits")
    .version("1.0.0");

  program
    .command("info")
    .description("view the location of your config file")
    .action(async () => {
      info(config);
    });

  program
    .command("configure")
    .description("set configuration options")
    .option(
      "--timezone <VALUE>",
      "Set a specific, fixed, timezone to prevent leaking your system timezone",
    )
    .option(
      "--allow-push-outside-timeslot",
      "Allow push command outside timeslots, may trigger CI runs",
    )
    .option("--no-allow-push-outside-timeslot")
    .action(async (options) => {
      process.exit(configure(config, options));
    });

  program
    .command("timeslot")
    .option(
      "-a, --add",
      "Add a timeslot, defined by --days, --start and --end",
      false,
    )
    .option(
      "--days <VALUE>",
      "Days this timeslot applies to, eg 1-5 for Monday through Friday or 6-7 for Saturday and Sunday",
    )
    .option("--start <VALUE>", "Start time, eg 0900 for 9am or 1730 for 5:30pm")
    .option("--end <VALUE>", "End time, eg 0900 for 9am or 1730 for 5:30pm")
    .option("-l, --list", "List timeslots", false)
    .description("manage timeslots in which to commit")
    .action(async (options) => {
      process.exit(timeslot(options, config));
    });

  program
    .command("commit")
    .allowUnknownOption(true)
    .helpOption(false)
    .description("run git commit with modified times")
    .action(async () => {
      process.exit(await commit(process.argv.slice(3), config));
    });

  program
    .command("push")
    .allowUnknownOption(true)
    .helpOption(false)
    .description("run git push ensuring no commits are in the future")
    .action(async () => {
      process.exit(await push(process.argv.slice(3), config));
    });

  program
    .command("rebase")
    .allowUnknownOption(true)
    .helpOption(false)
    .description("run git rebase with the pre-commit hook")
    .action(async () => {
      process.exit(await rebase(process.argv.slice(3), config));
    });

  program
    .command("rewrite-history")
    .helpOption(false)
    .description("rewrite git history with dates within timeslots")
    .action(async () => {
      process.exit(await rewriteHistory(config));
    });

  program
    .command("pre-commit-hook")
    .description("prevents mistakenly committing outside timeslots")
    .action(async () => {
      process.exit(await preCommitHook(config));
    });

  program
    .command("pre-push-hook")
    .description("prevents mistakenly pushing outside timeslots")
    .action(async () => {
      process.exit(await prePushHook(config));
    });

  program
    .command("pre-rebase-hook")
    .description("prevents mistakenly rebasing outside timeslots")
    .action(async () => {
      process.exit(await preRebaseHook(config));
    });

  program.parse(process.argv);
}

main();
