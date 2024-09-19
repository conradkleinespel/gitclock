const console = require("node:console");
const { getLastCommitDate } = require("../git");
const { DateTime } = require("luxon");

exports.preCommitHook = async function (config) {
  console.log("Running gitclock pre-commit-hook...");

  const timeslots = config.getTimeslots();
  if (!timeslots.length) {
    console.error("No timeslots found. Please add timeslots.");
    return 1;
  }

  const currentDate = DateTime.now();
  if (
    process.env.GIT_CLOCK !== "1" &&
    timeslots.filter((t) => t.isDateWithin(currentDate)).length === 0
  ) {
    console.error(
      "Cannot commit outside timeslot. Use gitclock to create your commit.",
    );
    return 1;
  }

  if (
    process.env.GIT_CLOCK !== "1" &&
    process.env.GIT_COMMITTER_DATE == null &&
    (await getLastCommitDate()) > currentDate
  ) {
    console.error(
      "Cannot commit with current date, because last commit is in the future. Use gitclock.",
    );
    return 1;
  }

  console.log("Pre-commit hook finished successfully.");
  return 0;
};
