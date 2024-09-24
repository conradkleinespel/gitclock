const console = require("node:console");
const fs = require("node:fs");
const { getPushObjectDate } = require("../git");
const { DateTime } = require("luxon");

async function prePushHook(config) {
  console.log("Running gitclock pre-push-hook...");

  const timeslots = config.getTimeslots();
  if (!timeslots.length) {
    console.error("Error: No timeslots found. Please add timeslots.");
    return 1;
  }

  const currentDate = DateTime.now();
  if (
    process.env.GITCLOCK !== "1" &&
    !config.getAllowPushOutsideTimeslot() &&
    timeslots.filter((t) => t.isDateWithin(currentDate)).length === 0
  ) {
    console.error(
      "Error: Cannot push outside timeslot. This could cause CI to trigger.",
    );
    return 1;
  }

  const input = fs.readFileSync(process.stdin.fd, "utf-8");
  if (!input.length) {
    // Nothing to push
    return 0;
  }

  const inputParts = input.split(" ");
  /* istanbul ignore if */
  if (inputParts.length !== 4) {
    throw new Error(
      `Unreachable. Pre-push hook input "${inputParts.join(" ")}" is invalid. Please report this as a bug.`,
    );
  }

  const localObjectName = inputParts[1];
  const localObjectDate = await getPushObjectDate(localObjectName);

  if (process.env.GITCLOCK !== "1" && localObjectDate > currentDate) {
    console.error(
      "Error: Trying to push commits that are in the future. Aborting.",
    );
    return 1;
  }

  console.log("Pre-push hook finished successfully.");
  return 0;
}

module.exports = { prePushHook };
