const console = require("node:console");
const { getLastCommitDate, gitCommit } = require("../git");
const { DateTime } = require("luxon");

const minTimeBetweenCommitsMinutes = 1;
const maxTimeBetweenCommitsMinutes = 15;

async function commit(args, config) {
  const timeslots = config.getTimeslots();
  if (!timeslots.length) {
    console.log("No timeslots found. Please add timeslots.");
    return 1;
  }

  const currentDate = DateTime.now();
  const lastCommitDate = await getLastCommitDate();
  const minDateForNextCommit =
    lastCommitDate > currentDate ? lastCommitDate : currentDate;
  const nextCommitDate = getNextCommitDate(
    currentDate,
    minDateForNextCommit,
    timeslots,
  );

  /* istanbul ignore if */
  if (nextCommitDate < currentDate) {
    throw new Error(
      "Unreachable. Next commit date must be in the present or future. Please report this as a bug.",
    );
  }

  return await gitCommit(nextCommitDate, config.getTimezone(), args);
}

function getNextCommitDate(currentDate, minDate, timeslots) {
  let nextCommitDate = null;
  for (let timeslot of timeslots) {
    let thisNextCommitDate = timeslot.nextSuitableDate(minDate);
    // We want the earliest possible commit slot
    if (!nextCommitDate) {
      nextCommitDate = thisNextCommitDate;
    } else if (nextCommitDate > thisNextCommitDate) {
      nextCommitDate = thisNextCommitDate;
    }
  }

  /* istanbul ignore if */
  if (!nextCommitDate) {
    throw new Error(
      "Unreachable. There are timeslots, there should be a next commit date. Please report this as a bug.",
    );
  }

  // If we are currently within schedule, we want to commit as if gitclock was not even there
  if (currentDate.toMillis() === nextCommitDate.toMillis()) {
    return currentDate;
  }

  return nextCommitDate.plus({
    minutes: Math.floor(
      Math.random() * maxTimeBetweenCommitsMinutes +
        minTimeBetweenCommitsMinutes,
    ),
    seconds: Math.max(
      0,
      Math.floor(Math.random() * (60 - nextCommitDate.second)) - 1,
    ),
  });
}

module.exports = {
  commit,
  getNextCommitDate,
};
