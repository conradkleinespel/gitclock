const console = require("node:console");
const { gitRebase } = require("../git");
const { DateTime } = require("luxon");

async function rebase(args, config) {
  const timeslots = config.getTimeslots();
  if (!timeslots.length) {
    console.log("No timeslots found. Please add timeslots.");
    return 1;
  }

  return await gitRebase(["--committer-date-is-author-date", ...args]);
}

module.exports = {
  rebase,
};
