const console = require("node:console");
const { DateTime } = require("luxon");

exports.preRebaseHook = async function (config) {
  console.log("Running gitclock pre-rebase-hook...");

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
    console.error("Cannot rebase outside timeslot. Use gitclock to rebase.");
    return 1;
  }

  console.log("Pre-rebase hook finished successfully.");
  return 0;
};
