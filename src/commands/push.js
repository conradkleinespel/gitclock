const console = require("node:console");
const {
  getFirstPastCommitHash,
  getTrackingRemoteAndBranch,
  gitPush,
} = require("../git");
const { DateTime } = require("luxon");

async function push(args, config) {
  const timeslots = config.getTimeslots();
  if (!timeslots.length) {
    console.log("No timeslots found. Please add timeslots.");
    return 1;
  }

  const currentDate = DateTime.now();
  if (
    !config.getAllowPushOutsideTimeslot() &&
    timeslots.filter((t) => t.isDateWithin(currentDate)).length === 0
  ) {
    console.warn(
      "Cannot push outside timeslot. This could cause CI to trigger.",
    );
    return 1;
  }

  const { commitHash: firstPastCommitHash, error: firstPastCommitHashError } =
    await getFirstPastCommitHash();
  if (firstPastCommitHashError) {
    console.log(`Error: ${firstPastCommitHashError}`);
    return 1;
  }

  const {
    remote,
    branch,
    error: trackingRemoteAndBranchError,
  } = await getTrackingRemoteAndBranch();
  if (trackingRemoteAndBranchError) {
    console.log(`Error: ${trackingRemoteAndBranchError}`);
    return 1;
  }

  return await gitPush([...args, remote, `${firstPastCommitHash}:${branch}`]);
}

module.exports = {
  push,
};
